# Expose-rs
This is very simple library for Port Forwarding in Rust.
I saw others' solutions, but they are kinda complicated and have tons of dependencies (that's not you'd like to see precisely in Rust).

## Usage
The library is with standard functions (not async, futures, tokio, etc.). If you're using async functions, this is may not your choice here... but you always can spawn threads!
Also, this library uses only WANIPConnection1 interface.

Everything in one example
```
    let session = discover().unwrap();
    forward_port(&session, "TCP", 6969, 6969, "test", 0).unwrap(); // forwards internal 6969 port to external 6969
    close_port(&session, "TCP", 6969).unwrap(); // closes external 6969 port
    get_extenral_ip(&session).unwrap(); // do I need to what that function does?
```