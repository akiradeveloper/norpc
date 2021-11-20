use super::Service;

fn generate_request(svc: &Service) -> String {
    let mut list = vec![];
    for fun in &svc.functions {
        let mut params = vec![];
        for param in &fun.inputs {
            params.push(param.typ_name.clone());
        }
        list.push(format!("{}({})", fun.name, itertools::join(params, ",")));
    }
    format!(
        "
        #[allow(non_camel_case_types)]
        pub enum {}Request {{
		{}
	}}",
        svc.name,
        itertools::join(list, ",")
    )
}
fn generate_response(svc: &Service) -> String {
    let mut list = vec![];
    for fun in &svc.functions {
        list.push(format!("{}({})", fun.name, fun.output));
    }
    format!(
        "
        #[allow(non_camel_case_types)]
        pub enum {}Response {{
		{}
	}}",
        svc.name,
        itertools::join(list, ",")
    )
}
fn generate_client_struct(svc: &Service) -> String {
    format!(
        "
    #[derive(Clone)]
	pub struct {}Client<Svc> {{
		svc: Svc
	}}
	",
        svc.name
    )
}
fn generate_server_struct(svc: &Service) -> String {
    format!(
        "
	#[derive(Clone)]
	pub struct {}Service<App: {}> {{
		app: App
	}}
	",
        svc.name, svc.name
    )
}
fn generate_trait(svc: &Service) -> String {
    let mut list = vec![];
    for fun in &svc.functions {
        let mut params = vec!["&self".to_owned()];
        for param in &fun.inputs {
            params.push(format!("{}:{}", param.var_name, param.typ_name));
        }
        let params = itertools::join(params, ",");
        list.push(format!(
            "async fn {}({}) -> Result<{}, Self::Error>;",
            fun.name, params, fun.output
        ));
    }
    format!(
        "
		#[norpc::async_trait]
		pub trait {}: Clone {{
			type Error;
			{}
		}}
		",
        svc.name,
        itertools::join(list, "")
    )
}
fn generate_client_impl(svc: &Service) -> String {
    let mut funlist = vec![];
    for fun in &svc.functions {
        let mut params = vec!["&self".to_owned()];
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
		async fn {}({}) -> Result<{}, norpc::Error<Svc::Error>> {{
			let rep = self.svc.clone().call({}Request::{}({})).await.map_err(norpc::Error::AppError)?;
			match rep {{
				{}Response::{}(v) => Ok(v),
				_ => unreachable!(),
			}}
		}}
		",
            fun.name, params, fun.output, svc.name, fun.name, req_params, svc.name, fun.name
        );
        funlist.push(f);
    }

    format!(
        "
	impl<Svc: Service<{}Request, Response = {}Response> + Clone> {}Client<Svc> {{
		pub fn new(svc: Svc) -> Self {{
			Self {{ svc }}
		}}
		{}
	}}
	",
        svc.name,
        svc.name,
        svc.name,
        itertools::join(funlist, "")
    )
}
fn generate_server_impl(svc: &Service) -> String {
    let mut arms = vec![];
    for fun in &svc.functions {
        let mut req_params = vec![];
        for p in &fun.inputs {
            req_params.push(p.var_name.to_owned());
        }
        let req_params = itertools::join(req_params, ",");

        let a = format!(
            "
		{}Request::{}({}) => {{
			let rep = app.{}({}).await?;
			Ok({}Response::{}(rep))
		}}
		",
            svc.name, fun.name, req_params, fun.name, req_params, svc.name, fun.name
        );

        arms.push(a);
    }
    let arms = itertools::join(arms, "");

    format!(
        "
	impl<App: {}> {}Service<App> {{
		pub fn new(app: App) -> Self {{
			Self {{ app }}
		}}
		pub async fn call(self, req: {}Request) -> Result<{}Response, App::Error> {{
			let app = self.app.clone();
			match req {{
				{}
			}}
		}}
	}}
	",
        svc.name, svc.name, svc.name, svc.name, arms
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
