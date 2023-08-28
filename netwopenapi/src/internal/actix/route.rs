use crate::internal::actix::utils::OperationUpdater;
use crate::internal::actix::METHODS;
use crate::path_item_definition::PathItemDefinition;
use actix_service::ServiceFactory;
use actix_web::dev::ServiceRequest;
use actix_web::guard::Guard;
use actix_web::http::Method;
use actix_web::{Error, FromRequest, Handler, Responder};
use std::collections::BTreeMap;
use utoipa::openapi::path::Operation;
use utoipa::openapi::{Components, PathItem, PathItemType};

/// Wrapper for [`actix_web::web::method`](https://docs.rs/actix-web/*/actix_web/web/fn.method.html).
pub fn method(method: Method) -> Route {
  Route::new().method(method)
}

/// Wrapper for [`actix_web::web::get`](https://docs.rs/actix-web/*/actix_web/web/fn.get.html).
pub fn get() -> Route {
  method(Method::GET)
}

/// Wrapper for [`actix_web::web::put`](https://docs.rs/actix-web/*/actix_web/web/fn.put.html).
pub fn put() -> Route {
  method(Method::PUT)
}

/// Wrapper for [`actix_web::web::post`](https://docs.rs/actix-web/*/actix_web/web/fn.post.html).
pub fn post() -> Route {
  method(Method::POST)
}

/// Wrapper for [`actix_web::web::patch`](https://docs.rs/actix-web/*/actix_web/web/fn.patch.html).
pub fn patch() -> Route {
  method(Method::PATCH)
}

/// Wrapper for [`actix_web::web::delete`](https://docs.rs/actix-web/*/actix_web/web/fn.delete.html).
pub fn delete() -> Route {
  method(Method::DELETE)
}

/// Wrapper for [`actix_web::web::options`](https://docs.rs/actix-web/*/actix_web/web/fn.options.html).
pub fn options() -> Route {
  method(Method::OPTIONS)
}

/// Wrapper for [`actix_web::web::head`](https://docs.rs/actix-web/*/actix_web/web/fn.head.html).
pub fn head() -> Route {
  method(Method::HEAD)
}

pub struct Route {
  operation: Option<Operation>,
  path_item_type: Option<PathItemType>,
  components: Vec<Components>,
  inner: actix_web::Route,
}

impl ServiceFactory<ServiceRequest> for Route {
  type Config = ();
  type Error = Error;
  type InitError = ();
  type Service = <actix_web::Route as ServiceFactory<ServiceRequest>>::Service;
  type Future = <actix_web::Route as ServiceFactory<ServiceRequest>>::Future;
  type Response =
    <<actix_web::Route as ServiceFactory<ServiceRequest>>::Service as actix_service::Service<ServiceRequest>>::Response;

  #[allow(clippy::unit_arg)]
  fn new_service(&self, cfg: Self::Config) -> Self::Future {
    self.inner.new_service(cfg)
  }
}

impl Route {
  /// Wrapper for [`actix_web::Route::new`](https://docs.rs/actix-web/*/actix_web/struct.Route.html#method.new)
  #[allow(clippy::new_without_default)]
  pub fn new() -> Route {
    Route {
      operation: None,
      path_item_type: None,
      components: Default::default(),
      inner: actix_web::Route::new(),
    }
  }

  /// Wrapper for [`actix_web::Route::method`](https://docs.rs/actix-web/*/actix_web/struct.Route.html#method.method)
  pub fn method(mut self, method: Method) -> Self {
    let path_item_type = match method.as_str() {
      "PUT" => PathItemType::Put,
      "POST" => PathItemType::Post,
      "DELETE" => PathItemType::Delete,
      "OPTIONS" => PathItemType::Options,
      "HEAD" => PathItemType::Head,
      "PATCH" => PathItemType::Patch,
      _ => PathItemType::Get,
    };
    self.path_item_type = Some(path_item_type);
    self.inner = self.inner.method(method);
    self
  }

  /// Proxy for [`actix_web::Route::guard`](https://docs.rs/actix-web/*/actix_web/struct.Route.html#method.guard).
  ///
  /// **NOTE:** This doesn't affect spec generation.
  pub fn guard<G: Guard + 'static>(mut self, guard: G) -> Self {
    self.inner = self.inner.guard(guard);
    self
  }

  /// Wrapper for [`actix_web::Route::to`](https://docs.rs/actix-web/*/actix_web/struct.Route.html#method.to)
  pub fn to<F, Args>(mut self, handler: F) -> Self
  where
    F: Handler<Args>,
    Args: FromRequest + 'static,
    F::Output: Responder + 'static,
    F::Future: PathItemDefinition,
  {
    if F::Future::is_visible() {
      self.operation = Some(F::Future::operation());
      self.components = F::Future::components();
    }
    self.inner = self.inner.to(handler);
    self
  }
}

pub(crate) struct PathDefinition {
  pub(crate) path: String,
  pub(crate) item: PathItem,
}

pub(crate) struct RouteWrapper {
  pub(crate) def: PathDefinition,
  pub(crate) component: Vec<Components>,
  pub(crate) inner: actix_web::Route,
}

impl RouteWrapper {
  pub(crate) fn new<S: Into<String>>(path: S, route: Route) -> Self {
    let mut operations: BTreeMap<PathItemType, Operation> = Default::default();
    let mut path_item = PathItem::default();
    let path: String = path.into();
    if let Some(mut operation) = route.operation {
      operation.update_path_parameter_name_from_path(&path);

      if let Some(path_item_type) = route.path_item_type {
        operations.insert(path_item_type, operation);
      } else {
        for path_item_type in METHODS {
          operations.insert(path_item_type.clone(), operation.clone());
        }
      }
    }
    path_item.operations = operations;

    Self {
      def: PathDefinition { path, item: path_item },
      component: route.components,
      inner: route.inner,
    }
  }
}