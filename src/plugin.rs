use std::time::{Duration, SystemTime};

use chrono::{DateTime, Local, Utc};
use nu_plugin::{EngineInterface, EvaluatedCall, Plugin, SimplePluginCommand};
use nu_protocol::{Example, LabeledError, Signature, Type, Value};
use ulid::Ulid;

pub struct UlidPlugin;

impl UlidPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for UlidPlugin {
    fn commands(&self) -> Vec<Box<dyn nu_plugin::PluginCommand<Plugin = Self>>> {
        vec![Box::new(RandomUlid), Box::new(ParseUlid)]
    }

    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }
}

pub struct RandomUlid;

impl SimplePluginCommand for RandomUlid {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "random ulid"
    }

    fn usage(&self) -> &str {
        "Generate a random ulid"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .search_terms(vec!["generate".into(), "ulid".into(), "uuid".into()])
            .input_output_types(vec![
                (Type::Nothing, Type::String),
                (Type::Date, Type::String),
                (
                    Type::Record(Box::new([
                        (K_TS.into(), Type::Date),
                        (K_RND.into(), Type::String),
                    ])),
                    Type::String,
                ),
                (
                    Type::Record(Box::new([
                        (K_TS.into(), Type::Date),
                        (K_RND.into(), Type::Int),
                    ])),
                    Type::String,
                ),
                (
                    Type::Record(Box::new([(K_TS.into(), Type::Date)])),
                    Type::String,
                ),
                (
                    Type::Record(Box::new([(K_RND.into(), Type::String)])),
                    Type::String,
                ),
                (
                    Type::Record(Box::new([(K_RND.into(), Type::Int)])),
                    Type::String,
                ),
            ])
            .switch(
                "zeroed",
                "Fill the random portion of the ulid with zeros",
                Some('0'),
            )
            .switch(
                "oned",
                "Fill the random portion of the ulid with ones",
                Some('1'),
            )
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Generate a random ulid based on the current time".into(),
                example: "random ulid".into(),
                result: Some(Value::test_string(Ulid::new().to_string())),
            },
            Example {
                description: "Generate a random ulid based on the given timestamp".into(),
                example: "2024-03-19T11:46:00 | random ulid".into(),
                result: Some(Value::test_string(
                    Ulid::from_datetime(
                        SystemTime::UNIX_EPOCH + Duration::from_nanos(1710848760000000000),
                    )
                    .to_string(),
                )),
            },
            Example {
                description:
                    "Generate a ulid based on the current time with the random portion all set to 0"
                        .into(),
                example: "random ulid --zeroed".into(),
                result: Some(Value::test_string(
                    Ulid::from_parts(unix_millis(None), 0).to_string(),
                )),
            },
        ]
    }

    fn run(
        &self,
        _plugin: &UlidPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let (timestamp, random): (Option<SystemTime>, UlidRandom) = match input {
            Value::Nothing { .. } => (None, self.selected_randomness(call, None)?),
            Value::Date { val, .. } => (Some((*val).into()), self.selected_randomness(call, None)?),
            Value::Record { val, .. } => (
                val.get(K_TS)
                    .map(|ts| ts.as_date())
                    .transpose()?
                    .map(|ts| ts.into()),
                self.selected_randomness(call, val.get(K_RND))?,
            ),
            _ => {
                return Err(LabeledError::new("Invalid input").with_label(
                    format!(
                        "Input type of {} is not supported",
                        input.get_type().to_string()
                    ),
                    input.span(),
                ))
            }
        };

        Ok(Value::string(
            self.generate(timestamp, random).to_string(),
            call.head,
        ))
    }
}

enum UlidRandom {
    Random,
    Set(u128),
    Zeros,
    Ones,
}

impl RandomUlid {
    fn selected_randomness(
        &self,
        call: &EvaluatedCall,
        input: Option<&Value>,
    ) -> Result<UlidRandom, LabeledError> {
        match (
            call.has_flag("zeroed").unwrap(),
            call.has_flag("oned").unwrap(),
            input,
        ) {
            (true, true, _) => Err(LabeledError::new("Flag error")
                .with_label("Cannot set --zeroed and --oned at the same time", call.head)),
            (true, false, _) => Ok(UlidRandom::Zeros),
            (false, true, _) => Ok(UlidRandom::Ones),
            (false, false, None) => Ok(UlidRandom::Random),
            (false, false, Some(input)) => match input {
                Value::String { val, internal_span } => {
                    Ok(UlidRandom::Set(val.parse::<u128>().map_err(|e| {
                        LabeledError::new("Invalid number")
                            .with_label(e.to_string(), *internal_span)
                    })?))
                }
                Value::Int { val, .. } => Ok(UlidRandom::Set(*val as u128)),
                _ => Err(LabeledError::new("Invalid number").with_label(
                    format!(
                        "{} is not a valid number",
                        input.to_abbreviated_string(&nu_protocol::Config::default())
                    ),
                    input.span(),
                )),
            },
        }
    }

    fn generate(&self, timestamp: Option<SystemTime>, random: UlidRandom) -> Ulid {
        match (timestamp, random) {
            (None, UlidRandom::Random) => Ulid::new(),
            (Some(ts), UlidRandom::Random) => Ulid::from_datetime(ts),
            (ts, UlidRandom::Set(r)) => Ulid::from_parts(unix_millis(ts), r),
            (ts, UlidRandom::Zeros) => Ulid::from_parts(unix_millis(ts), 0),
            (ts, UlidRandom::Ones) => Ulid::from_parts(unix_millis(ts), u128::MAX),
        }
    }
}

fn unix_millis(timestamp: Option<SystemTime>) -> u64 {
    timestamp
        .unwrap_or_else(SystemTime::now)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub struct ParseUlid;

static K_TS: &str = "timestamp";
static K_RND: &str = "random";

impl SimplePluginCommand for ParseUlid {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "parse ulid"
    }

    fn usage(&self) -> &str {
        "Parse a ulid into a date"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .search_terms(vec!["parse".into(), "ulid".into(), "date".into()])
            .input_output_types(vec![(
                Type::String,
                Type::Record(Box::new([
                    (K_TS.into(), Type::Date),
                    (K_RND.into(), Type::String),
                ])),
            )])
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Generate a ulid and parse out the date portion".into(),
            example: "random ulid | parse ulid | get timestamp".into(),
            result: Some(Value::test_date(Local::now().fixed_offset())),
        }]
    }

    fn run(
        &self,
        _plugin: &UlidPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let ulid: Ulid = input.coerce_str()?.parse::<Ulid>().map_err(|e| {
            LabeledError::new("Failed to parse ulid").with_label(e.to_string(), input.span())
        })?;

        let date: DateTime<Utc> = ulid.datetime().into();
        let date = Value::date(date.fixed_offset(), call.head);
        Ok(Value::record(
            [
                (K_TS.into(), date),
                (
                    K_RND.into(),
                    Value::string(ulid.random().to_string(), call.head),
                ),
            ]
            .into_iter()
            .collect(),
            call.head,
        ))
    }
}
