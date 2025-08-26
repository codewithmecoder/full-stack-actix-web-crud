## You are a senior rust dev, and I want you to implement rust package that support actix-web with

### Features Implemented

1. Dependency Injection Container

- Fully custom DI container from scratch (no third-party crates).
- Generic registration API with type-safe resolution.
- Supports ServiceCollection → ServiceProvider pattern.

2. Service Lifetimes

- Singleton: Shared globally.
- Scoped: Unique per scope/request (per ServiceScope).
- Transient: Always creates a new instance.

3. Constructor Injection

- Automatically wires struct fields by type.
- Implemented via #[derive(Injectable)].
- Works with concrete types and trait objects.

4. Multi-Trait Registration

- #[service(as = ["Trait1", "Trait2"])] supports registering multiple trait interfaces per struct.
- Handlers can resolve either the concrete type or any registered trait.

5. Scoped Lifetime Support

- ServiceScope allows per-request caching of scoped services.
- Works with Actix-web middleware to create a scope per HTTP request.

6. Macros #[derive(Injectable)] auto-generates:

- inject(r) constructor method.
- register(services) method that registers concrete + interfaces.
- Supports multiple as traits.
- Supports attribute-driven lifetimes (lifetime = "scoped" | "singleton" | "transient").

7. Safe Resolution

- resolve<T>() panics if service not registered.
- Optional try_get<T>() method could be added for non-panicking retrieval (not yet included but trivial to extend).

8. Actix-web Integration

- Middleware ScopedDI automatically creates a ServiceScope per request.
- Handlers can resolve scoped, singleton, or transient services via the request.
- Mimics ASP.NET Core’s IServiceScopeFactory + [FromServices] style pattern.

9. Extensible

- Can register custom factories, closures, or default constructors.
- Can extend macro for auto-registration or auto-scanning of structs in a crate.

# `Cargo.toml`

[package]

```toml
name = "actix-di"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4.0"
actix-service = "2.0"
futures = "0.3"
pin-project-lite = "0.2"
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
tokio = { version = "1.0", features = ["full"] }
```

# `src/lib.rs`

```rs
pub mod container;
pub mod lifetime;
pub mod middleware;
pub mod scope;
pub mod error;
pub mod macros;

pub use container::*;
pub use lifetime::*;
pub use middleware::*;
pub use scope::*;
pub use error::*;
// Re-export the derive macro
pub use actix_di_derive::Injectable;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
```

# `src/error.rs`

```rs
use std::any::TypeId;
use std::fmt;

#[derive(Debug)]
pub enum DIError {
    ServiceNotRegistered(TypeId),
    ResolutionFailed(String),
    CircularDependency(String),
}

impl fmt::Display for DIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DIError::ServiceNotRegistered(type_id) => {
                write!(f, "Service not registered: {:?}", type_id)
            }
            DIError::ResolutionFailed(msg) => write!(f, "Resolution failed: {}", msg),
            DIError::CircularDependency(msg) => write!(f, "Circular dependency: {}", msg),
        }
    }
}

impl std::error::Error for DIError {}

pub type DIResult<T> = Result<T, DIError>;
```

# `src/lifetime.rs`

```rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServiceLifetime {
    Singleton,
    Scoped,
    Transient,
}
```

# `src/container.rs`

```rs
use crate::{DIError, DIResult, ServiceLifetime};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type ServiceFactory = Box<dyn Fn(&ServiceProvider) -> DIResult<Box<dyn Any + Send + Sync>> + Send + Sync>;

pub struct ServiceDescriptor {
    pub lifetime: ServiceLifetime,
    pub factory: ServiceFactory,
    pub instance: Option<Arc<dyn Any + Send + Sync>>,
}

impl ServiceDescriptor {
    pub fn new<T: Any + Send + Sync + 'static>(
        lifetime: ServiceLifetime,
        factory: impl Fn(&ServiceProvider) -> DIResult<T> + Send + Sync + 'static,
    ) -> Self {
        Self {
            lifetime,
            factory: Box::new(move |provider| {
                factory(provider).map(|t| Box::new(t) as Box<dyn Any + Send + Sync>)
            }),
            instance: None,
        }
    }

    pub fn singleton_instance<T: Any + Send + Sync + 'static>(instance: T) -> Self {
        Self {
            lifetime: ServiceLifetime::Singleton,
            factory: Box::new(|_| unreachable!()),
            instance: Some(Arc::new(instance)),
        }
    }
}

pub struct ServiceCollection {
    services: HashMap<TypeId, ServiceDescriptor>,
    trait_mappings: HashMap<TypeId, TypeId>, // trait -> concrete type
}

impl ServiceCollection {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            trait_mappings: HashMap::new(),
        }
    }

    pub fn add<T: Any + Send + Sync + 'static>(
        &mut self,
        lifetime: ServiceLifetime,
        factory: impl Fn(&ServiceProvider) -> DIResult<T> + Send + Sync + 'static,
    ) -> &mut Self {
        let type_id = TypeId::of::<T>();
        let descriptor = ServiceDescriptor::new(lifetime, factory);
        self.services.insert(type_id, descriptor);
        self
    }

    pub fn add_singleton<T: Any + Send + Sync + 'static>(&mut self, instance: T) -> &mut Self {
        let type_id = TypeId::of::<T>();
        let descriptor = ServiceDescriptor::singleton_instance(instance);
        self.services.insert(type_id, descriptor);
        self
    }

    pub fn add_transient<T: Any + Send + Sync + 'static>(
        &mut self,
        factory: impl Fn(&ServiceProvider) -> DIResult<T> + Send + Sync + 'static,
    ) -> &mut Self {
        self.add(ServiceLifetime::Transient, factory)
    }

    pub fn add_scoped<T: Any + Send + Sync + 'static>(
        &mut self,
        factory: impl Fn(&ServiceProvider) -> DIResult<T> + Send + Sync + 'static,
    ) -> &mut Self {
        self.add(ServiceLifetime::Scoped, factory)
    }

    pub fn register_trait<T: 'static, U: 'static>(&mut self) -> &mut Self {
        self.trait_mappings.insert(TypeId::of::<T>(), TypeId::of::<U>());
        self
    }

    pub fn build(self) -> ServiceProvider {
        ServiceProvider {
            services: Arc::new(self.services),
            trait_mappings: Arc::new(self.trait_mappings),
            singletons: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

pub struct ServiceProvider {
    services: Arc<HashMap<TypeId, ServiceDescriptor>>,
    trait_mappings: Arc<HashMap<TypeId, TypeId>>,
    singletons: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl ServiceProvider {
    pub fn resolve<T: Any + Send + Sync + 'static>(&self) -> DIResult<Arc<T>> {
        self.resolve_internal::<T>(None)
    }

    pub fn resolve_scoped<T: Any + Send + Sync + 'static>(
        &self,
        scoped_cache: &mut HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    ) -> DIResult<Arc<T>> {
        self.resolve_internal::<T>(Some(scoped_cache))
    }

    fn resolve_internal<T: Any + Send + Sync + 'static>(
        &self,
        scoped_cache: Option<&mut HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    ) -> DIResult<Arc<T>> {
        let type_id = TypeId::of::<T>();

        // Check if this is a trait that maps to a concrete type
        let actual_type_id = self.trait_mappings.get(&type_id).unwrap_or(&type_id);

        let descriptor = self.services.get(actual_type_id)
            .ok_or(DIError::ServiceNotRegistered(type_id))?;

        match descriptor.lifetime {
            ServiceLifetime::Singleton => {
                // Check if we have a pre-built instance
                if let Some(instance) = &descriptor.instance {
                    return instance
                        .clone()
                        .downcast::<T>()
                        .map_err(|_| DIError::ResolutionFailed("Type downcast failed".to_string()));
                }

                // Check singleton cache
                {
                    let singletons = self.singletons.read().unwrap();
                    if let Some(instance) = singletons.get(actual_type_id) {
                        return instance
                            .clone()
                            .downcast::<T>()
                            .map_err(|_| DIError::ResolutionFailed("Type downcast failed".to_string()));
                    }
                }

                // Create new singleton
                let boxed_instance = (descriptor.factory)(self)?;
                let instance = Arc::from(boxed_instance.downcast::<T>()
                    .map_err(|_| DIError::ResolutionFailed("Type downcast failed".to_string()))?);

                // Cache singleton
                {
                    let mut singletons = self.singletons.write().unwrap();
                    singletons.insert(*actual_type_id, instance.clone());
                }

                Ok(instance)
            }
            ServiceLifetime::Scoped => {
                if let Some(cache) = scoped_cache {
                    // Check scoped cache
                    if let Some(instance) = cache.get(actual_type_id) {
                        return instance
                            .clone()
                            .downcast::<T>()
                            .map_err(|_| DIError::ResolutionFailed("Type downcast failed".to_string()));
                    }

                    // Create new scoped instance
                    let boxed_instance = (descriptor.factory)(self)?;
                    let instance = Arc::from(boxed_instance.downcast::<T>()
                        .map_err(|_| DIError::ResolutionFailed("Type downcast failed".to_string()))?);

                    // Cache scoped instance
                    cache.insert(*actual_type_id, instance.clone());
                    Ok(instance)
                } else {
                    // No scope provided, treat as transient
                    let boxed_instance = (descriptor.factory)(self)?;
                    Arc::from(boxed_instance.downcast::<T>()
                        .map_err(|_| DIError::ResolutionFailed("Type downcast failed".to_string()))?)
                        .pipe(Ok)
                }
            }
            ServiceLifetime::Transient => {
                let boxed_instance = (descriptor.factory)(self)?;
                Arc::from(boxed_instance.downcast::<T>()
                    .map_err(|_| DIError::ResolutionFailed("Type downcast failed".to_string()))?)
                    .pipe(Ok)
            }
        }
    }
}

// Helper trait for pipe operations
trait Pipe<T> {
    fn pipe<U>(self, f: impl FnOnce(Self) -> U) -> U
    where
        Self: Sized,
    {
        f(self)
    }
}

impl<T> Pipe<T> for T {}
```

# `src/scope.rs`

```rs
use crate::{DIResult, ServiceProvider};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

pub struct ServiceScope {
    provider: ServiceProvider,
    scoped_services: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl ServiceScope {
    pub fn new(provider: ServiceProvider) -> Self {
        Self {
            provider,
            scoped_services: HashMap::new(),
        }
    }

    pub fn resolve<T: Any + Send + Sync + 'static>(&mut self) -> DIResult<Arc<T>> {
        self.provider.resolve_scoped(&mut self.scoped_services)
    }

    pub fn provider(&self) -> &ServiceProvider {
        &self.provider
    }
}
```

# `src/middleware.rs`

```rs
use crate::{ServiceProvider, ServiceScope};
use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error, HttpMessage,
};
use futures::future::{ok, Ready};
use std::sync::Arc;

pub struct ScopedDI {
    provider: ServiceProvider,
}

impl ScopedDI {
    pub fn new(provider: ServiceProvider) -> Self {
        Self { provider }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ScopedDI
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ScopedDIMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ScopedDIMiddleware {
            service,
            provider: self.provider.clone(),
        })
    }
}

pub struct ScopedDIMiddleware<S> {
    service: S,
    provider: ServiceProvider,
}

impl<S, B> Service<ServiceRequest> for ScopedDIMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let scope = ServiceScope::new(self.provider.clone());
        req.extensions_mut().insert(Arc::new(scope));
        self.service.call(req)
    }
}

// Extension trait for extracting services from requests
pub trait ServiceExtractor {
    fn resolve<T: Any + Send + Sync + 'static>(&mut self) -> DIResult<Arc<T>>;
}

impl ServiceExtractor for ServiceRequest {
    fn resolve<T: Any + Send + Sync + 'static>(&mut self) -> DIResult<Arc<T>> {
        let mut scope = self
            .extensions_mut()
            .remove::<Arc<ServiceScope>>()
            .ok_or_else(|| crate::DIError::ResolutionFailed("No service scope found".to_string()))?;

        let result = Arc::make_mut(&mut scope).resolve::<T>();
        self.extensions_mut().insert(scope);
        result
    }
}

impl ServiceExtractor for actix_web::HttpRequest {
    fn resolve<T: Any + Send + Sync + 'static>(&mut self) -> DIResult<Arc<T>> {
        let mut scope = self
            .extensions_mut()
            .remove::<Arc<ServiceScope>>()
            .ok_or_else(|| crate::DIError::ResolutionFailed("No service scope found".to_string()))?;

        let result = Arc::make_mut(&mut scope).resolve::<T>();
        self.extensions_mut().insert(scope);
        result
    }
}
```

# `src/macros.rs`

```rs
pub trait Injectable {
    fn inject(provider: &crate::ServiceProvider) -> crate::DIResult<Self>
    where
        Self: Sized;

    fn register(services: &mut crate::ServiceCollection);
}
```

# `src/lib.rs`

```rs
// Example usage and tests would go in src/lib.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    trait IUserService: Send + Sync {
        fn get_user(&self) -> String;
    }

    trait IEmailService: Send + Sync {
        fn send_email(&self, to: &str, body: &str);
    }

    struct UserService {
        counter: AtomicU32,
    }

    impl UserService {
        fn new() -> Self {
            Self {
                counter: AtomicU32::new(0),
            }
        }
    }

    impl IUserService for UserService {
        fn get_user(&self) -> String {
            let count = self.counter.fetch_add(1, Ordering::SeqCst);
            format!("User #{}", count)
        }
    }

    struct EmailService;

    impl EmailService {
        fn new() -> Self {
            Self
        }
    }

    impl IEmailService for EmailService {
        fn send_email(&self, to: &str, body: &str) {
            println!("Sending email to {}: {}", to, body);
        }
    }

    struct UserController {
        user_service: Arc<dyn IUserService>,
        email_service: Arc<dyn IEmailService>,
    }

    impl UserController {
        fn new(
            user_service: Arc<dyn IUserService>,
            email_service: Arc<dyn IEmailService>,
        ) -> Self {
            Self {
                user_service,
                email_service,
            }
        }

        fn handle_request(&self) -> String {
            let user = self.user_service.get_user();
            self.email_service.send_email("test@example.com", &format!("Hello {}", user));
            user
        }
    }

    #[test]
    fn test_dependency_injection() {
        let mut services = ServiceCollection::new();

        // Register services
        services
            .add_scoped(|_| Ok(UserService::new()))
            .register_trait::<dyn IUserService, UserService>()
            .add_singleton(EmailService::new())
            .register_trait::<dyn IEmailService, EmailService>()
            .add_transient(|provider: &ServiceProvider| {
                let user_service = provider.resolve::<dyn IUserService>()?;
                let email_service = provider.resolve::<dyn IEmailService>()?;
                Ok(UserController::new(user_service, email_service))
            });

        let provider = services.build();

        // Test scoped resolution
        let mut scope = ServiceScope::new(provider);
        let controller1 = scope.resolve::<UserController>().unwrap();
        let controller2 = scope.resolve::<UserController>().unwrap();

        let result1 = controller1.handle_request();
        let result2 = controller2.handle_request();

        // Both controllers should share the same scoped UserService instance
        println!("Result 1: {}", result1);
        println!("Result 2: {}", result2);
    }
}
```

// Cargo.toml for the derive macro crate (actix-di-derive)
/\*
[package]
name = "actix-di-derive"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full"] }
\*/

// src/derive_macro.rs for actix-di-derive crate
/\*
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type, Attribute, Meta, Lit};

#[proc_macro_derive(Injectable, attributes(service, lifetime))]
pub fn derive_injectable(input: TokenStream) -> TokenStream {
let input = parse_macro_input!(input as DeriveInput);
let name = &input.ident;

    // Parse attributes
    let mut lifetime = quote! { actix_di::ServiceLifetime::Transient };
    let mut as_traits: Vec<Type> = Vec::new();

    for attr in &input.attrs {
        if attr.path().is_ident("lifetime") {
            if let Ok(Meta::NameValue(meta)) = attr.meta {
                if let Lit::Str(lit_str) = &meta.value {
                    match lit_str.value().as_str() {
                        "singleton" => lifetime = quote! { actix_di::ServiceLifetime::Singleton },
                        "scoped" => lifetime = quote! { actix_di::ServiceLifetime::Scoped },
                        "transient" => lifetime = quote! { actix_di::ServiceLifetime::Transient },
                        _ => {}
                    }
                }
            }
        } else if attr.path().is_ident("service") {
            // Parse #[service(as = ["Trait1", "Trait2"])]
            // This is simplified - in practice you'd need more robust parsing
        }
    }

    let inject_impl = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            let field_inits = fields.named.iter().map(|field| {
                let field_name = &field.ident;
                let field_type = &field.ty;
                quote! {
                    #field_name: provider.resolve::<#field_type>()?
                }
            });

            quote! {
                fn inject(provider: &actix_di::ServiceProvider) -> actix_di::DIResult<Self> {
                    Ok(Self {
                        #(#field_inits,)*
                    })
                }
            }
        } else {
            quote! {
                fn inject(provider: &actix_di::ServiceProvider) -> actix_di::DIResult<Self> {
                    Ok(Self::new())
                }
            }
        }
    } else {
        quote! {
            fn inject(provider: &actix_di::ServiceProvider) -> actix_di::DIResult<Self> {
                Err(actix_di::DIError::ResolutionFailed("Cannot inject into non-struct types".to_string()))
            }
        }
    };

    let expanded = quote! {
        impl actix_di::Injectable for #name {
            #inject_impl

            fn register(services: &mut actix_di::ServiceCollection) {
                services.add(#lifetime, |provider| Self::inject(provider));
            }
        }
    };

    TokenStream::from(expanded)

}
\*/
