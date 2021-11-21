use super::Service;

fn generate_request(svc: &Service) -> String {
    let mut variants = vec![];
    for fun in &svc.functions {
        let mut params = vec![];
        for param in &fun.inputs {
            params.push(param.typ_name.clone());
        }
        variants.push(format!("{}({})", fun.name, itertools::join(params, ",")));
    }
    format!(
        "
        #[allow(non_camel_case_types)]
        pub enum {}Request {{
		{}
	}}",
        svc.name,
        itertools::join(variants, ",")
    )
}
fn generate_response(svc: &Service) -> String {
    let mut variants = vec![];
    for fun in &svc.functions {
        variants.push(format!("{}({})", fun.name, fun.output));
    }
    format!(
        "
        #[allow(non_camel_case_types)]
        pub enum {svc_name}Response {{
		{}
	}}",
        itertools::join(variants, ","),
        svc_name = svc.name,
    )
}
fn generate_client_struct(svc: &Service) -> String {
    format!(
        "
    #[derive(Clone)]
	pub struct {svc_name}Client<Svc> {{
		svc: Svc
	}}
    pub type {svc_name}ClientT = {svc_name}Client<norpc::ClientChannel<{svc_name}Request, {svc_name}Response>>;
	",
        svc_name = svc.name
    )
}
fn generate_server_struct(svc: &Service) -> String {
    format!(
        "
	#[derive(Clone)]
	pub struct {svc_name}Service<App: {svc_name}> {{
		app: App
	}}
	",
        svc_name = svc.name,
    )
}
fn generate_trait(svc: &Service) -> String {
    let mut methods = vec![];
    for fun in &svc.functions {
        let mut params = vec!["self".to_owned()];
        for param in &fun.inputs {
            params.push(format!("{}:{}", param.var_name, param.typ_name));
        }
        let params = itertools::join(params, ",");
        methods.push(format!(
            "async fn {}({}) -> Result<{}, Self::Error>;",
            fun.name, params, fun.output
        ));
    }
    format!(
        "
		#[norpc::async_trait]
		pub trait {svc_name}: Clone {{
			type Error;
			{}
		}}
		",
        itertools::join(methods, ""),
        svc_name = svc.name,
    )
}
fn generate_client_impl(svc: &Service) -> String {
    let mut methods = vec![];
    for fun in &svc.functions {
        let mut params = vec!["&mut self".to_owned()];
        for p in &fun.inputs {
            params.push(format!("{}:{}", p.var_name, p.typ_name));
        }
        let params = itertools::join(params, ",");

        let mut req_params = vec![];
        for p in &fun.inputs {
            req_params.push(p.var_name.to_owned());
        }
        let req_params = itertools::join(req_params, ",");

        let f = format!(
            "
		async fn {fun_name}({params}) -> Result<{output}, norpc::Error<Svc::Error>> {{
			let rep = self.svc.call({}Request::{fun_name}({req_params})).await.map_err(norpc::Error::AppError)?;
			match rep {{
				{svc_name}Response::{fun_name}(v) => Ok(v),
                #[allow(unreachable_patterns)]
				_ => unreachable!(),
			}}
		}}
		",
            svc_name = svc.name,
            fun_name = fun.name,
            params = params,
            output = fun.output,
            req_params = req_params,
        );
        methods.push(f);
    }
    format!(
        "
	impl<Svc: Service<{svc_name}Request, Response = {svc_name}Response>> {svc_name}Client<Svc> {{
		pub fn new(svc: Svc) -> Self {{
			Self {{ svc }}
		}}
		{}
	}}
	",
        itertools::join(methods, ""),
        svc_name = svc.name,
    )
}
fn generate_server_impl(svc: &Service) -> String {
    let mut match_arms = vec![];
    for fun in &svc.functions {
        let mut req_params = vec![];
        for p in &fun.inputs {
            req_params.push(p.var_name.to_owned());
        }
        let req_params = itertools::join(req_params, ",");

        let a = format!(
            "
		{svc_name}Request::{fun_name}({req_params}) => {{
			let rep = app.{fun_name}({req_params}).await?;
			Ok({svc_name}Response::{fun_name}(rep))
		}}
		",
            svc_name = svc.name,
            fun_name = fun.name,
            req_params = req_params,
        );

        match_arms.push(a);
    }

    format!(
        "
	impl<App: {svc_name}> {svc_name}Service<App> {{
		pub fn new(app: App) -> Self {{
			Self {{ app }}
		}}
	}}
    impl<App: {svc_name} + 'static + Send> tower::Service<{svc_name}Request> for {svc_name}Service<App> {{
        type Response = {svc_name}Response;
        type Error = App::Error;
        type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;
        fn poll_ready(
            &mut self,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {{
            Ok(()).into()
        }}
		fn call(&mut self, req: {svc_name}Request) -> Self::Future {{
			let app = self.app.clone();
            Box::pin(async move {{
                match req {{
                    {}
                }}
            }})
		}}
    }}
	",
        itertools::join(match_arms, ","),
        svc_name = svc.name,
    )
}
pub(super) fn generate(services: Vec<Service>) -> String {
    let mut out = String::new();
    for svc in services {
        let s = format!(
            "{}{}{}{}{}{}{}",
            generate_request(&svc),
            generate_response(&svc),
            generate_trait(&svc),
            generate_client_struct(&svc),
            generate_client_impl(&svc),
            generate_server_struct(&svc),
            generate_server_impl(&svc),
        );
        out.push_str(&s);
    }
    out
}
