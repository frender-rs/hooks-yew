use std::{cell::RefCell, pin::Pin, rc::Rc, task::Poll};

use hooks::{dyn_hook, HookPollNextUpdate};
use yew::{Component, Html, Properties};

pub type DynHookComponent<Props> = dyn_hook![ for<'a> (&'a Props) -> Html ];

pub struct PinBoxDynHookComponent<Props> {
    hook: RefCell<Pin<Box<DynHookComponent<Props>>>>,
    abort_control: Option<Rc<()>>,
}

impl<Props> PinBoxDynHookComponent<Props> {
    #[inline]
    pub fn new(hook: Pin<Box<DynHookComponent<Props>>>) -> Self {
        Self {
            hook: RefCell::new(hook),
            abort_control: None,
        }
    }
}

impl<Props: 'static + Properties> PinBoxDynHookComponent<Props> {
    pub fn view(&self, props: &Props) -> Html {
        let mut hook = self.hook.borrow_mut();

        let hook = hook.as_mut();

        hook.erased_use_hook((props,))
    }

    pub fn rendered<COMP: Component<Message = bool>>(
        &mut self,
        ctx: &yew::Context<COMP>,
        mut get_hook: impl FnMut(&COMP) -> &Self + 'static,
    ) {
        let weak = if let Some(ac) = &self.abort_control {
            if Rc::weak_count(ac) == 0 && Rc::strong_count(ac) == 1 {
                Some(Rc::downgrade(ac))
            } else {
                None
            }
        } else {
            let abort_control = Rc::new(());
            let weak = Rc::downgrade(&abort_control);
            self.abort_control = Some(abort_control);
            Some(weak)
        };

        if let Some(mut weak) = weak {
            let scope = ctx.link();
            let s = scope.clone();

            scope.send_future(std::future::poll_fn(move |cx| {
                if weak.strong_count() == 0 {
                    return Poll::Ready(false);
                }

                let comp = s.get_component().unwrap();
                let mut hook = get_hook(&comp).hook.borrow_mut();
                let hook = hook.as_mut();
                let res =
                    <DynHookComponent<Props> as HookPollNextUpdate>::poll_next_update(hook, cx);

                if res.is_ready() {
                    std::mem::take(&mut weak);
                }

                res
            }));
        }
    }

    pub fn erased_changed(&mut self) -> bool {
        let _ = self.abort_control.take();
        true
    }
}
