use std::time::{Duration, SystemTime};

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
        vec![RandomUlid.signature()]
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
        let timestamp = if input.is_nothing() {
            None
        } else {
            Some(input.as_date()?.into())
        };
        let padding = self.padding(call)?;

        Ok(Value::string(
            self.generate(timestamp, padding).to_string(),
            call.head,
        ))
    }
}

enum UlidPadding {
    Random,
    Zeros,
    Ones,
}

impl RandomUlid {
    fn padding(&self, call: &EvaluatedCall) -> Result<UlidPadding, LabeledError> {
        match (
            call.has_flag("zeroed").unwrap(),
            call.has_flag("oned").unwrap(),
        ) {
            (true, true) => Err(LabeledError {
                label: "Flag error".into(),
                msg: "Cannot set --zeroed and --oned at the same time".into(),
                span: Some(call.head),
            }),
            (true, false) => Ok(UlidPadding::Zeros),
            (false, true) => Ok(UlidPadding::Ones),
            (false, false) => Ok(UlidPadding::Random),
        }
    }

    fn generate(&self, timestamp: Option<SystemTime>, padding: UlidPadding) -> Ulid {
        match (timestamp, padding) {
            (None, UlidPadding::Random) => Ulid::new(),
            (Some(ts), UlidPadding::Random) => Ulid::from_datetime(ts),
            (ts, UlidPadding::Zeros) => Ulid::from_parts(unix_millis(ts), 0),
            (ts, UlidPadding::Ones) => Ulid::from_parts(unix_millis(ts), u128::MAX),
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
