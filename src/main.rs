use nu_plugin::{serve_plugin, MsgPackSerializer};
use nu_plugin_ulid::UlidPlugin;

fn main() {
    serve_plugin(&mut UlidPlugin::new(), MsgPackSerializer {});
}
