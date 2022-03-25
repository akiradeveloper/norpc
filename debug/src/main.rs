#[norpc::service]
trait HelloWorld {
    fn hello(s: String) -> String;
}

fn main() {
	
}