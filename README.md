# rust-handlers

A simple compiler plugin for generating systems of callback handlers.

# Usage

To generate a system, use the `handlers_define_system!` macro:

```rust
handlers_define_system! <system name> {
    <handler name> {
        <signal>(<args>) => <slot>;
        ...
    }
    ...
}
```

This defines a system struct, an object trait, and a handler trait for each defined handler in the system.
The system will have each signal as a method, which will call the appropriate slot for each object of that handler type it contains.
The object trait is special, and is used to convert each object in the system to the correct trait type.
To add objects to the system, implement whatever handlers you want and then use the `handlers_impl_object!` macro to provide the correct object trait implementation:

```rust
handlers_impl_object! <system name> {
    <object name>: <handler name>, ...
}
```

To see a better usage example, see the test folder in this repository.
