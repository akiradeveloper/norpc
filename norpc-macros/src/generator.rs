use super::Service;

pub struct Generator {
    pub no_send: bool,
}
impl Generator {
    fn generate_request(&self, svc: &Service) -> String {
        let mut variants = vec![];
        for fun in &svc.functions {
            let mut params = vec![];
            for param in &fun.inputs {
                params.push(param.typ_name.clone());
            }
            variants.push(format!("{}({})", fun.name, &itertools::join(params, ","),));
        }
        format!(
            "
        #[allow(non_camel_case_types)]
        pub enum {svc_name}Request {{
		{}
	}}",
            itertools::join(variants, ","),
            svc_name = svc.name,
        )
    }
    fn generate_response(&self, svc: &Service) -> String {
        let mut variants = vec![];
        for fun in &svc.functions {
            variants.push(format!("{}({})", fun.name, fun.output,));
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
    fn generate_client_struct(&self, svc: &Service) -> String {
        format!(
            "
    #[derive(Clone)]
	pub struct {svc_name}Client<Svc> {{
		svc: Svc
	}}
	",
            svc_name = svc.name,
        )
    }
    fn generate_server_struct(&self, svc: &Service) -> String {
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
    fn generate_trait(&self, svc: &Service) -> String {
        let mut methods = vec![];
        for fun in &svc.functions {
            let mut params = vec!["self".to_owned()];
            for param in &fun.inputs {
                params.push(format!("{}:{}", param.var_name, param.typ_name,));
            }
            let params = itertools::join(params, ",");
            methods.push(format!(
                "async fn {}({}) -> {};",
                fun.name, &params, fun.output,
            ));
        }
        format!(
            "
		#[norpc::async_trait{no_send}]
		pub trait {svc_name}: Clone {{
			{}
		}}
		",
            itertools::join(methods, ""),
            svc_name = svc.name,
            no_send = if self.no_send { "(?Send)" } else { "" },
        )
    }
    fn generate_client_impl(&self, svc: &Service) -> String {
        let mut methods = vec![];
        for fun in &svc.functions {
            let mut params = vec!["&mut self".to_owned()];
            for p in &fun.inputs {
                params.push(format!("{}:{}", p.var_name, p.typ_name,));
            }
            let params = itertools::join(params, ",");

            let mut req_params = vec![];
            for p in &fun.inputs {
                req_params.push(p.var_name.to_owned());
            }
            let req_params = itertools::join(req_params, ",");

            let f = format!(
                "
		pub async fn {fun_name}({params}) -> std::result::Result<{output}, Svc::Error> {{
            norpc::poll_fn(|ctx| self.svc.poll_ready(ctx)).await.ok();
			let rep = self.svc.call({}Request::{fun_name}({req_params})).await?;
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
	impl<Svc: norpc::Service<{svc_name}Request, Response = {svc_name}Response>> {svc_name}Client<Svc> {{
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
    fn generate_server_impl(&self, svc: &Service) -> String {
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
			let rep = app.{fun_name}({req_params}).await;
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
    impl<App: {svc_name} + 'static {no_send}> norpc::Service<{svc_name}Request> for {svc_name}Service<App> {{
        type Response = {svc_name}Response;
        type Error = ();
        type Future = std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<Self::Response, Self::Error>> {no_send}>>;
        fn poll_ready(
            &mut self,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::result::Result<(), Self::Error>> {{
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
        no_send = if self.no_send { "" } else { "+ Send" },
    )
    }
    pub(super) fn generate(&self, svc: Service) -> String {
        format!(
            "{}{}{}{}{}{}{}",
            self.generate_request(&svc),
            self.generate_response(&svc),
            self.generate_trait(&svc),
            self.generate_client_struct(&svc),
            self.generate_client_impl(&svc),
            self.generate_server_struct(&svc),
            self.generate_server_impl(&svc),
        )
    }
}
