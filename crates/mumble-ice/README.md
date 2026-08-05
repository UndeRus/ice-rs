# mumble-ice

Человеческий API к Murmur (Mumble server) через ZeroC Ice.

```rust
use mumble_ice::prelude::*;

#[tokio::main]
async fn main() -> mumble_ice::Result<()> {
    let client = MurmurClient::connect("127.0.0.1:6502").await?;
    let srv = client.only_server().await?;

    for user in srv.users().await? {
        println!("{} в канале {}", user.name, user.channel);
    }
    srv.broadcast("привет").await?;
    Ok(())
}
```

## Что этот слой убирает

Работа со сгенерированным Slice-кодом напрямую выглядит так:

```rust
let mut comm = Communicator::new().await?;
let proxy = comm.string_to_proxy("Meta:tcp -h 127.0.0.1 -p 6502").await?;
let mut meta = MetaPrx::unchecked_cast(proxy).await?;
let mut major = 0; let mut minor = 0; let mut patch = 0; let mut text = String::new();
meta.get_version(&mut major, &mut minor, &mut patch, &mut text, ctx.clone()).await?;
```

| Было | Стало |
|---|---|
| `Option<HashMap<String,String>>` с секретом в каждом вызове | `.secret(...)` один раз на билдере |
| `&mut self` на каждом методе | `&self` + `Clone`, хендл раздаётся по таскам |
| out-параметры как `&mut` | именованные структуры (`Version`, `AclSnapshot`) |
| `session` и `userid` оба `i32` | `SessionId` / `UserId` — перепутать не скомпилируется |
| `userid == -1` | `Option<UserId>` |
| `-1`/`-2` из `verifyPassword` | `PasswordCheck` |
| пустая строка из `getConf` | `Option<String>` |
| `onlinesecs: i32` | `Duration` |
| `address: Vec<u8>` (16 байт) | `Option<IpAddr>` |
| `version` + `version2` | `ClientVersion` |
| `userid == -1 ? group` в ACL | `AclSubject::{User, Group}` |
| `0x01` руками | `Permission::WRITE` (`bitflags`) |
| `Acllist`, `Dbstate` | `Vec<Acl>`, `DbState` |
| `Box<dyn Error>` и разбор по тексту | `Error` с вариантами + `is_transient()` / `is_stale_handle()` |
| `Vec<Box<Tree>>` | `ChannelTree` с `walk()` / `find()` / `find_by_path()` |

## Read-modify-write

`setState`, `setChannelState`, `setACL` и `setBans` отправляют структуру
**целиком**, поэтому собирать её с нуля значит затирать чужие изменения. Для этого
есть хелперы:

```rust
srv.update_user(session, |u| { u.mute = true; u.channel = ChannelId(3); }).await?;
srv.update_acl(ChannelId::ROOT, |acl| {
    acl.allow(AclSubject::Group("admin".into()), Permission::WRITE);
}).await?;
```

`add_ban` и `remove_bans` тоже сделаны через чтение-правку-запись, потому что в
Slice есть только «заменить весь список».

## Колбеки

```rust
struct Bot;

#[async_trait::async_trait]
impl ServerEvents for Bot {
    async fn user_connected(&self, srv: &VirtualServer, u: User) -> mumble_ice::Result<()> {
        srv.message_user(u.session, &format!("привет, {}!", u.name)).await
    }
}

let sub = srv.on_events(Arc::new(Bot)).await?;
sub.closed().await;   // разрешится, если подписка умрёт
```

У всех методов трейта есть дефолт, поэтому реализуется только нужное. Состояние
бота лежит в его структуре рядом с обработчиками — не нужен `Arc` на каждое
замыкание. Есть и `srv.events()` — тот же мост, но потоком, для главных циклов на
`select!`.

Что фасад делает сам:

- **Переподписка после перезапуска.** `MumbleServer.ice` предупреждает: остановка
  виртуального сервера снимает колбеки. Фасад ставит внутренний `MetaCallback`,
  возвращает подписки и вызывает `reattached()`. После него все закэшированные
  `SessionId` — мусор, поэтому это отдельный метод, а не строчка в логе.
- **Ошибки и паники не глотаются.** Исключение из колбека заставляет Murmur молча
  снять регистрацию целиком, поэтому Murmur'у мы отвечаем «ок» всегда, а `Err` и
  паника обработчика уходят в `on_error()`. Подписка это переживает — проверено
  тестом.
- **Адрес обратного вызова.** Murmur звонит наружу, поэтому под Docker/NAT:

  ```rust
  MurmurClient::builder()
      .host_port("murmur", 6502)
      .callback_listen("0.0.0.0:7100".parse()?)   // внутри контейнера
      .callback_advertise("bot", 7100)            // как Murmur нас найдёт
      .connect().await?
  ```

  Wildcard без явного `advertise` — ошибка на `connect()`, а не невнятный
  `InvalidCallback` секундами позже.

## Аутентификатор

Один трейт на **оба** Slice-интерфейса (`ServerAuthenticator` и
`ServerUpdatingAuthenticator`), у всех методов дефолт «не моё» — минимальный
рабочий аутентификатор это одна реализация `authenticate`.

```rust
#[async_trait::async_trait]
impl Authenticator for MapAuth {
    async fn authenticate(&self, req: AuthRequest) -> AuthResult {
        let acct = match self.by_name.get(&req.name_ci()) {
            Some(a) => a,
            // Незнакомое имя — FallThrough, а НЕ Denied.
            None => return AuthResult::FallThrough,
        };
        if acct.password != req.password {
            return AuthResult::Denied;
        }
        AuthResult::Ok(AuthOk::new(acct.id).rename("Alice [staff]").group("admin"))
    }
}

let sub = srv.set_authenticator(Arc::new(auth)).await?;
```

Сентинелы Murmur'а наружу не выходят:

| Slice | Фасад |
|---|---|
| `authenticate` → id / `-1` / `-2` / `-3` | `AuthResult::{Ok, Denied, FallThrough, Unavailable}` |
| `getInfo` → `bool` + out | `Lookup<UserInfo>` |
| `nameToId` → id / `-2` | `Lookup<UserId>` |
| `idToName` → имя / `""` | `Lookup<String>` |
| `idToTexture` → байты / пусто | `Lookup<Vec<u8>>` |
| `registerUser` → id / `-1` / `-2` | `RegisterResult` |
| `unregisterUser`, `setInfo`, `setTexture` → `1`/`0`/`-1` | `UpdateResult` |

**`Denied` против `FallThrough`** — самое важное во всём модуле. `Denied` значит
«пароль неверный», `FallThrough` — «имени не знаю, спроси свою базу». Перепутать
их значит заблокировать всех пользователей из базы Murmur'а. `Lookup` намеренно
не `Option` и конвертируется только явным `from_option`: неявное `None → Unknown`
это ровно тот способ, которым проваливаются в fall-through случайно.

Ещё две вещи:

- **`VirtualServer` в трейт не передаётся.** Обратный вызов в `Server`/`Meta`
  из аутентификатора вешает Murmur (`MumbleServer.ice` предупреждает об этом
  прямо). Здесь это запрещено типами.
- **Паника деградирует, а не блокирует.** На провод уходит безопасное значение:
  fall-through для поисков, `Unavailable` для `authenticate`. Сломанный
  аутентификатор превращается в «Murmur смотрит свою базу», а не в «никто не
  может войти».

Проверить, не поднимая Mumble-клиента: **`Server::verifyPassword` идёт прямо в
`authenticate`** (установлено диагностическим прогоном). Заодно выяснилось, что
`getRegistration` и `getTexture` идут в `getInfo`, а `getRegisteredUsers` — в
`getRegisteredUsers`.

## Escape hatch

Обёрнута основная часть операций. Длинный хвост — через `raw()`, но с уже готовым
контекстом:

```rust
use mumble_ice::slice::mumble_server::Server as _;

let mut raw = srv.raw().await;
let ctx = raw.ctx();
let conf = raw.get_all_conf(ctx).await?;
```

## Конкурентность

Вызовы к одному серверу идут **параллельно** по одному соединению: запросы
мультиплексируются по `request_id`, ответы разбирает reader-таск. Хендлы `Clone`
и `&self`, поэтому раздаются по таскам без внешнего мьютекса:

```rust
let (users, channels) = tokio::join!(srv.users(), srv.channels());
```

Работает и на `current_thread`-рантайме. Проверено фактом, а не замером времени:
тест смотрит пик одновременных запросов на соединении (`5` при пяти вызовах).

## Как гонять E2E

Murmur не переживает `Meta::getServer` и остановку/запуск виртуального сервера в
пределах одного процесса (см. ниже), поэтому одним прогоном весь набор не идёт —
его надо разбивать, перезапуская сервер между группами:

```bash
cargo nextest run -p mumble-ice --test e2e --run-ignored all
```

```bash
cargo nextest run -p mumble-ice --test e2e_events --run-ignored all -E 'not test(reattaches)'
```

```bash
cargo nextest run -p mumble-ice --test e2e_events --run-ignored all -E 'test(reattaches)'
```

```bash
cargo nextest run -p mumble-ice --test e2e_auth --run-ignored all
```

Аутентификатор у виртуального сервера может быть только один, поэтому тесты
`e2e_auth` сериализованы группой в `.config/nextest.toml` — параллельно они
перебивали бы друг другу регистрацию.

## Замеченные повадки Murmur 1.5.857

Проверено на живом сервере, в документации Slice этого нет:

- `getAssumedDatabaseState` объявлен в Slice, но **не реализован** — приходит
  `OperationNotExist`. Отдаётся как `Error::OperationNotSupported`, чтобы бот мог
  деградировать, а не падать.
- `getLog(first, last)` — **полуинтервал** `[first, last)`: при `getLogLen() == 142`
  вызов `getLog(0, 141)` отдаёт 141 запись.
- Исключения приезжают со строковым type-id, но с **нулевыми битами типа** во
  флагах слайса. Кто ветвится по битам — теряет тип исключения.
- `Meta::getServer` перемежающимся образом **валит сервер** (наблюдалось на
  x86_64-сборке под Rosetta): в лог уходит
  `QSqlDatabasePrivate::database: requested database does not belong to the
  calling thread`, затем фатальная ошибка на
  `SELECT server_id FROM servers`. Это дефект потоков Qt/SQL внутри Murmur, не
  клиента. Практический вывод: предпочитайте `only_server()` и
  `booted_servers()` — они идут через `getBootedServers` и этот путь не задевают.
- `getTree` кодирует классы **компактными type-id**, а не строковыми. Декодеру
  реестр типов не нужен (конкретный тип известен из сигнатуры операции), но
  наивная реализация на этом спотыкается.
