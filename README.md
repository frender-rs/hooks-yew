# hooks-yew

Compile-time hooks component for [yew](https://yew.rs/),
implemented with [`hooks`](https://github.com/frender-rs/hooks).

_This crate is still in alpha._

Add `hooks` and `hooks-yew` to your project.

```sh
cargo add hooks hooks-yew
```

```rust
use yew::prelude::*;

use hooks::use_state;
use hooks_yew::hook_component;

#[hook_component]
fn Counter() {
    let (state, updater) = use_state::<i32>(0);
    let updater = updater.clone();

    html! {
        <div>
            <button onclick={move |_| updater.replace_with_fn_pointer(|v| *v + 1)}>{ "+1" }</button>
            {state}
        </div>
    }
}

fn main() {
    yew::start_app::<Counter>();
}
```

Specify props with an argument:

```rust
#[hook_component]
fn Counter(props: Props) {
    // ...
}
```

Specify return type explicitly. (It must be `yew::Html`)

```rust
#[hook_component]
fn Counter(props: Props) -> ::yew::Html {
    // ...
}
```
