use std::time::{Duration, SystemTime};

use chrono::{DateTime, Local, Utc};
use nu_plugin::{EvaluatedCall, LabeledError, Plugin};
use nu_protocol::{PluginExample, PluginSignature, Type, Value};
use ulid::Ulid;

pub struct UlidPlugin;

impl UlidPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for UlidPlugin {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![RandomUlid.signature(), ParseUlid.signature()]
    }

    fn run(
        &mut self,
        name: &str,
        _config: &Option<Value>,
        call: &nu_plugin::EvaluatedCall,
        input: &Value,
    ) -> Result<Value, nu_plugin::LabeledError> {
        match name {
            "random ulid" => RandomUlid.run(self, (), call, input),
            "parse ulid" => ParseUlid.run(self, (), call, input),
            _ => Err(LabeledError {
                label: "Plugin call with wrong name signature".into(),
                msg: "the signature used to call the plugin does not match any name in the plugin signature vector".into(),
                span: Some(call.head),
            })
        }
    }
}

pub struct RandomUlid;

impl RandomUlid {
    fn signature(&self) -> PluginSignature {
        PluginSignature::build("random ulid")
            .usage("Generate a random ulid")
            .search_terms(vec!["generate".into(), "ulid".into(), "uuid".into()])
            .input_output_types(vec![
                (Type::Nothing, Type::String),
                (Type::Date, Type::String),
                (
                    Type::Record(vec![
                        (K_TS.into(), Type::Date),
                        (K_RND.into(), Type::String),
                    ]),
                    Type::String,
                ),
                (
                    Type::Record(vec![
                        (K_TS.into(), Type::Date),
                        (K_RND.into(), Type::Int),
                    ]),
                    Type::String,
                ),
                (Type::Record(vec![(K_TS.into(), Type::Date)]), Type::String),
                (Type::Record(vec![(K_RND.into(), Type::String)]), Type::String),
                (Type::Record(vec![(K_RND.into(), Type::Int)]), Type::String),
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
            .plugin_examples(vec![
                PluginExample {
                    description: "Generate a random ulid based on the current time".into(),
                    example: "random ulid".into(),
                    result: Some(Value::test_string(Ulid::new().to_string())),
                },
                PluginExample {
                    description: "Generate a random ulid based on the given timestamp".into(),
                    example: "2024-03-19T11:46:00 | random ulid".into(),
                    result: Some(Value::test_string(
                        Ulid::from_datetime(
                            SystemTime::UNIX_EPOCH + Duration::from_nanos(1710848760000000000),
                        )
                        .to_string(),
                    )),
                },
                PluginExample {
                    description: "Generate a ulid based on the current time with the random portion all set to 0".into(),
                    example: "random ulid --zeroed".into(),
                    result: Some(Value::test_string(Ulid::from_parts(unix_millis(None), 0).to_string())),
                },
            ])
    }

    fn run(
        &self,
        _plugin: &UlidPlugin,
        _engine: (),
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
                return Err(LabeledError {
                    label: "Invalid input".into(),
                    msg: format!(
                        "Input type of {} is not supported",
                        input.get_type().to_string()
                    ),
                    span: Some(input.span()),
                })
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
            (true, true, _) => Err(LabeledError {
                label: "Flag error".into(),
                msg: "Cannot set --zeroed and --oned at the same time".into(),
                span: Some(call.head),
            }),
            (true, false, _) => Ok(UlidRandom::Zeros),
            (false, true, _) => Ok(UlidRandom::Ones),
            (false, false, None) => Ok(UlidRandom::Random),
            (false, false, Some(input)) => match input {
                Value::String { val, internal_span } => {
                    Ok(UlidRandom::Set(val.parse::<u128>().map_err(|e| {
                        LabeledError {
                            label: "Invalid number".into(),
                            msg: e.to_string(),
                            span: Some(*internal_span),
                        }
                    })?))
                }
                Value::Int { val, .. } => Ok(UlidRandom::Set(*val as u128)),
                _ => Err(LabeledError {
                    label: "Invalid number".into(),
                    msg: format!(
                        "{} is not a valid number",
                        input.to_abbreviated_string(&nu_protocol::Config::default())
                    ),
                    span: Some(input.span()),
                }),
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

impl ParseUlid {
    fn signature(&self) -> PluginSignature {
        PluginSignature::build("parse ulid")
            .usage("Parse a ulid into a date")
            .search_terms(vec!["parse".into(), "ulid".into(), "date".into()])
            .input_output_types(vec![(
                Type::String,
                Type::Record(vec![
                    (K_TS.into(), Type::Date),
                    (K_RND.into(), Type::String),
                ]),
            )])
            .plugin_examples(vec![PluginExample {
                description: "Generate a ulid and parse out the date portion".into(),
                example: "random ulid | parse ulid | get timestamp".into(),
                result: Some(Value::test_date(Local::now().fixed_offset())),
            }])
    }

    fn run(
        &self,
        _plugin: &UlidPlugin,
        _engine: (),
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let ulid: Ulid = input
            .coerce_str()?
            .parse::<Ulid>()
            .map_err(|e| LabeledError {
                label: "Failed to parse ulid".into(),
                msg: e.to_string(),
                span: Some(input.span()),
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
