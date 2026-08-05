# murmur-slice

Биндинги Ice для Murmur (Mumble server), сгенерированные из `MumbleServer.ice`.
Рукописного кода в крейте нет.

Это низкий уровень: сырые `MetaPrx`/`ServerPrx`, servant-трейты `*I`, диспатчеры
`*Server` и Slice-типы. Человеческий API — слоем выше.

## Почему вывод закоммичен

Раньше код генерировался в `build.rs` при каждой сборке и лежал в `.gitignore`.
Следствия: каждому потребителю нужен `rustfmt` на `PATH`, сборка зависит от
разбора `.ice`, а изменения биндингов не видны в ревью. Отдельно неприятное: код
возврата `rustfmt` не проверялся, поэтому при сбое на диск молча уезжал **пустой**
`mod.rs`.

Теперь вывод в git, а генерация — осознанный ручной шаг.

## Регенерация

После правки `MumbleServer.ice`:

```bash
cargo run -p ice-rs --bin slice2rs -- crates/murmur-slice/src/gen crates/murmur-slice/MumbleServer.ice -i crates/murmur-slice/include
```

Затем посмотреть дифф и закоммитить. Вывод детерминирован: повторный запуск без
правок `.ice` не даёт изменений, и CI это проверяет.

## Что генерируется

| Из Slice | В Rust |
|---|---|
| `interface X` | трейт `X` (клиент), `XI` (servant), `XServer` (диспатчер), `XPrx` (прокси) |
| `struct` / `class` | структура с `ToBytes`/`FromBytes` |
| `enum` | `#[repr(i32)]` enum с `TryFromPrimitive` |
| `exception` | структура с `Display` + `std::error::Error` |
| `sequence` / `dictionary` | `pub type` на `Vec` / `HashMap` |
| `const int` | `pub const` в SCREAMING_SNAKE_CASE (`PermissionWrite` → `PERMISSION_WRITE`) |

Наследование интерфейсов сплющивается: `ServerUpdatingAuthenticator` получает и
свои пять операций, и пять унаследованных от `ServerAuthenticator`, а
`ice_type_ids()` отдаёт всю цепочку — без неё `checkedCast` к базовому типу со
стороны Murmur'а провалится.

## Известные ограничения

Инструмент покрывает то, что использует `MumbleServer.ice`, и не более:

- аббревиатуры в именах не распознаются, поэтому `ACLList` → `Acllist`,
  `DBState` → `Dbstate` (Inflector не умеет иначе); красивые имена — забота
  верхнего слоя;
- вложенные генерики (`dictionary<int, sequence<string>>`) грамматика не берёт;
- `optional`/tagged-члены реализованы частично (Murmur их не использует);
- у классов не поддержаны compact/index type-id (задето только `getTree`);
- `extends` принимает только простое имя, без `Module::Name`.
