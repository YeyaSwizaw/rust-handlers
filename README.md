# rust-handlers

A simple macro generating systems of event handlers.

# Usage

This macro uses the plugin [interpolate\_idents](https://crates.io/crates/interpolate_idents) and makes use
of the specialization feature, so currently requires rustc nightly.

To generate a system, use the `handlers!` macro:

```rust
handlers_define_system! {
    <system name> {
        <handler name> {
            <signal>(<args>) => <slot>;
            ...
        }
        ...
    }
}
```

This defines a system struct, an object trait, and a handler trait for each defined handler in the system.
The system will have each signal as a method, which will call the appropriate slot for each object of that handler type it contains.
The object trait is special, and is used to convert each object in the system to the correct trait type.

To create objects for the system, simply have them implement whatever handler traits you want them to, and then use the 
`handlers_objects!` macro. This will implement an object trait for them, and then specialization will work out the rest
of the required implementations

```rust
handlers_objects! { 
    <system name> {
        <object name>, ...
    }
}
```

To see a better usage example, see the test folder in this repository.
