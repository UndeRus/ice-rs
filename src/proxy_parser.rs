use std::hash::Hash;

use crate::{errors::*, protocol::{EndPointType, EndpointData}};
use pest::{Parser, iterators::Pairs};


#[derive(Parser)]
#[grammar = "proxystring.pest"]
pub struct ProxyParser;



pub struct DirectProxyData {
    pub ident: String,
    pub endpoint: EndPointType,
}

pub struct IndirectProxyData {
    pub ident: String,
    pub adapter: Option<String>,
}

pub enum ProxyStringType {
    DirectProxy(DirectProxyData),
    IndirectProxy(IndirectProxyData)
}

pub fn parse_proxy_string(proxy_string: &str) -> Result<ProxyStringType, Box<dyn std::error::Error + Sync + Send>> {
    let result = ProxyParser::parse(Rule::proxystring, proxy_string)?.next().unwrap();
    for child in result.into_inner() {
        match child.as_rule() {
            Rule::direct_proxy => return parse_direct_proxy(child.into_inner()),
            Rule::indirect_proxy => return parse_indirect_proxy(child.into_inner()),
            _ => {}
        }
    }
    Err(Box::new(ParsingError::new("Unexpected rule while parsing proxy string.")))
}

pub fn parse_direct_proxy(rules: Pairs<Rule>) -> Result<ProxyStringType, Box<dyn std::error::Error + Sync + Send>> {
    let mut ident = "";
    for child in rules {
        match child.as_rule() {
            Rule::ident => {
                ident = child.as_str();
            },
            Rule::endpoint => {
                return Ok(
                    ProxyStringType::DirectProxy(
                        DirectProxyData {
                            ident: String::from(ident),
                            endpoint: parse_endpoint(child.into_inner())?
                        }
                    )
                )
            }
            _ => {}
        }
    }
    Err(Box::new(ParsingError::new("Unexpected rule while parsing proxy string.")))
}

pub fn parse_indirect_proxy(rules: Pairs<Rule>) -> Result<ProxyStringType, Box<dyn std::error::Error + Sync + Send>> {
    let mut ident = "";
    let mut adapter = None;

    for child in rules {
        match child.as_rule() {
            Rule::ident => {
                ident = child.as_str();
            },
            Rule::adapter => {
                for child in child.into_inner() {
                    match child.as_rule() {
                        Rule::keyword_at => {}
                        Rule::ident => {
                            adapter = Some(child.as_str())
                        },
                        _ => return Err(Box::new(ParsingError::new("Unexpected rule while parsing proxy string.")))
                    }
                }
            },
            _ => return Err(Box::new(ParsingError::new("Unexpected rule while parsing proxy string.")))
        }
    }

    Ok(
        ProxyStringType::IndirectProxy(IndirectProxyData {
            ident: String::from(ident),
            adapter: if adapter.is_some() { Some(String::from(adapter.unwrap())) } else { None }
        })
    )

}

pub fn parse_endpoint(rules: Pairs<Rule>) -> Result<EndPointType, Box<dyn std::error::Error + Sync + Send>> {
    let mut protocol = "";
    let mut host = "";
    let mut port = 0i32;

    for child in rules {
        match child.as_rule() {
            Rule::endpoint_protocol => {
                protocol = child.as_str();
            }
            Rule::endpoint_host | Rule::endpoint_port => {
                for item in child.into_inner() {
                    match item.as_rule() {
                        Rule::hostname | Rule::ip => {
                            host = item.as_str().trim();
                        }
                        Rule::port => {
                            let parsed: i32 = item.as_str().parse()?;
                            // Грамматика допускает до пяти цифр, то есть 99999.
                            // Ноль разрешён: для привязки адаптера это штатный
                            // запрос «выдай любой свободный порт».
                            if parsed > 65535 {
                                return Err(Box::new(ParsingError::new(&format!(
                                    "Port {} out of range 0..=65535",
                                    parsed
                                ))));
                            }
                            port = parsed;
                        }
                        _ => return Err(Box::new(ParsingError::new(&format!("Unexpected proxy string rule: {:?}", item.as_rule()))))
                    };
                }
            }
            _ => return Err(Box::new(ParsingError::new("Unexpected rule while parsing proxy string.")))
        }
    }

    let endpoint_data = EndpointData {
        host: String::from(host),
        port,
        timeout: 60000,
        compress: false
    };

    match protocol {
        "tcp" | "default" => return Ok(EndPointType::TCP(endpoint_data)),
        "ssl" => return Ok(EndPointType::SSL(endpoint_data)),
        _ => return Err(Box::new(ParsingError::new("Unsupported protocol.")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::EndPointType;

    #[test]
    fn direct_proxy_parses_tcp_with_spaces() {
        match parse_proxy_string("Meta:tcp -h 127.0.0.1 -p 6502").unwrap() {
            ProxyStringType::DirectProxy(d) => {
                assert_eq!(d.ident, "Meta");
                match d.endpoint {
                    EndPointType::TCP(ep) => {
                        assert_eq!(ep.host, "127.0.0.1");
                        assert_eq!(ep.port, 6502);
                    }
                    _ => panic!("expected TCP"),
                }
            }
            _ => panic!("expected direct"),
        }
    }

    /// Правило hostname отвергало точки и цифры, поэтому ни одно реальное
    /// DNS-имя не парсилось — только односложные метки вроде `localhost`.
    #[test]
    fn dotted_hostnames_and_digits_parse() {
        for host in ["mumble.example.com", "host1", "localhost", "my-box.local"] {
            let s = format!("Meta:tcp -h {} -p 6502", host);
            match parse_proxy_string(&s).unwrap_or_else(|e| panic!("{}: {}", host, e)) {
                ProxyStringType::DirectProxy(d) => match d.endpoint {
                    EndPointType::TCP(ep) => assert_eq!(ep.host, host),
                    _ => panic!("expected TCP for {}", host),
                },
                _ => panic!("expected direct for {}", host),
            }
        }
    }

    #[test]
    fn single_digit_port_parses_and_out_of_range_rejected() {
        assert!(parse_proxy_string("Meta:tcp -h localhost -p 7").is_ok());
        // Порт 0 — валидный запрос эфемерного порта при привязке адаптера.
        assert!(parse_proxy_string("Cb:tcp -h 127.0.0.1 -p 0").is_ok());
        assert!(
            parse_proxy_string("Meta:tcp -h localhost -p 99999").is_err(),
            "порт вне 0..=65535 должен отвергаться"
        );
    }

    #[test]
    fn ssl_endpoint_parses() {
        match parse_proxy_string("Meta:ssl -h murmur.example.com -p 6502").unwrap() {
            ProxyStringType::DirectProxy(d) => match d.endpoint {
                EndPointType::SSL(ep) => {
                    assert_eq!(ep.host, "murmur.example.com");
                    assert_eq!(ep.port, 6502);
                }
                _ => panic!("expected SSL"),
            },
            _ => panic!("expected direct"),
        }
    }
}