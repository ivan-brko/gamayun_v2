//this ugly bit of code is needed to import generated rust code from proto files
//by doing it like this, we decide in which module we want to import generated code
pub mod gamayun {
    tonic::include_proto!("gamayun"); // The string specified here must match the proto package name
}
