#[macro_export]
macro_rules! lewis {
    (
        #[acidic(
            $QueryEvent:ident, $QueryOutput:ident,
            $UpdateEvent:ident, $UpdateOutput:ident,
            $AcidExt:ident
        )]
        impl $State:ident {
            $($body:tt)*
        }
    ) => {
        __lewis_parse! {
            @start
            $State
            ($QueryEvent $QueryOutput $UpdateEvent $UpdateOutput $AcidExt)
            ($($body)*)
        }
    };
}

#[macro_export]
macro_rules! __lewis_parse {
    // Initialize the loop
    (
        @start
        $State:ident
        ($($types:ident)*)
        (
            // Extract a "self" identifier hygenically
            fn $method:ident(&$self_:ident $($args:tt)*)
            $($rest:tt)*
        )
    ) => {
        __lewis_parse! {
            @parse
            $State
            ($($types)*)
            $self_
            ()
            ()
            (
                fn $method(&$self_ $($args)*)
                $($rest)*
            )
        }
    };
    (
        @start
        $State:ident
        ($($types:ident)*)
        (
            // Extract a "self" identifier hygenically
            fn $method:ident(&mut $self_:ident $($args:tt)*)
            $($rest:tt)*
        )
    ) => {
        __lewis_parse! {
            @parse
            $State
            ($($types)*)
            $self_
            ()
            ()
            (
                fn $method(&mut $self_ $($args)*)
                $($rest)*
            )
        }
    };
    (
        @start
        $State:ident
        ($($types:ident)*)
        ()
    ) => {
        __lewis_parse! {
            @parse
            $State
            ($($types)*)
            // If the user didn't list any methods, then we don't have any
            // "self" idents to use. So we make up our own.
            self
            ()
            ()
            ($($input)*)
        }
    };
    // Parse `&self` methods
    (
        @parse
        $State:ident
        ($($types:ident)*)
        $self_:ident
        ($($query_events:tt)*)
        ($($update_events:tt)*)
        (
            fn $method:ident(&self $(, $arg:ident: $Arg:ty)* $(,)*) -> $Out:ty {
                $($body:tt)*
            }
            $($rest:tt)*
        )
    ) => {
        __lewis_parse! {
            @parse
            $State
            ($($types)*)
            $self_
            (
                $($query_events)*
                ($method ($($arg: $Arg),*) ($Out) ($($body)*))
            )
            ($($update_events)*)
            ($($rest)*)
        }
    };
    // Parse `&self` methods that omit their return type
    (
        @parse
        $State:ident
        ($($types:ident)*)
        $self_:ident
        ($($query_events:tt)*)
        ($($update_events:tt)*)
        (
            fn $method:ident(&self $(, $arg:ident: $Arg:ty)* $(,)*) {
                $($body:tt)*
            }
            $($rest:tt)*
        )
    ) => {
        __lewis_parse! {
            @parse
            $State
            ($($types)*)
            $self_
            (
                $($query_events)*
                ($method ($($arg: $Arg),*) (()) ($($body)*))
            )
            ($($update_events)*)
            ($($rest)*)
        }
    };
    // Parse `&mut self` methods
    (
        @parse
        $State:ident
        ($($types:ident)*)
        $self_:ident
        ($($query_events:tt)*)
        ($($update_events:tt)*)
        (
            fn $method:ident(&mut self $(, $arg:ident: $Arg:ty)* $(,)*) -> $Out:ty {
                $($body:tt)*
            }
            $($rest:tt)*
        )
    ) => {
        __lewis_parse! {
            @parse
            $State
            ($($types)*)
            $self_
            ($($query_events)*)
            (
                $($update_events)*
                ($method ($($arg: $Arg),*) ($Out) ($($body)*))
            )
            ($($rest)*)
        }
    };
    // Parse `&mut self` methods that omit their return type
    (
        @parse
        $State:ident
        ($($types:ident)*)
        $self_:ident
        ($($query_events:tt)*)
        ($($update_events:tt)*)
        (
            fn $method:ident(&mut self $(, $arg:ident: $Arg:ty)* $(,)*) {
                $($body:tt)*
            }
            $($rest:tt)*
        )
    ) => {
        __lewis_parse! {
            @parse
            $State
            ($($types)*)
            $self_
            ($($query_events)*)
            (
                $($update_events)*
                ($method ($($arg: $Arg),*) (()) ($($body)*))
            )
            ($($rest)*)
        }
    };
    // Base case
    (
        @parse
        $State:ident
        ($QueryEvent:ident $QueryOutput:ident $UpdateEvent:ident $UpdateOutput:ident $AcidExt:ident)
        $self_:ident
        ($($query_events:tt)*)
        ($($update_events:tt)*)
        ()
    ) => {
        // Generate Acidic impl
        __lewis_impl_acidic! {
            $State
            ($QueryEvent $QueryOutput $UpdateEvent $UpdateOutput)
            $self_
            ($($query_events)*)
            ($($update_events)*)
        }

        // Create event and output enums
        __lewis_create_enums! {
            ($QueryEvent $QueryOutput)
            ($($query_events)*)
        }
        __lewis_create_enums! {
            ($UpdateEvent $UpdateOutput)
            ($($update_events)*)
        }

        // Generate extension trait
        __lewis_ext_trait! {
            $State
            ($QueryEvent $QueryOutput $UpdateEvent $UpdateOutput)
            $AcidExt
            $self_
            ($($query_events)*)
            ($($update_events)*)
        }
    };
}

#[macro_export]
macro_rules! __lewis_impl_acidic {
    (
        $State:ident
        ($QueryEvent:ident $QueryOutput:ident $UpdateEvent:ident $UpdateOutput:ident)
        $self_:ident
        ($($query_events:tt)*)
        ($($update_events:tt)*)
    ) => {
        impl $crate::Acidic for $State {
            type QueryEvent = $QueryEvent;
            type QueryOutput = $QueryOutput;
            type UpdateEvent = $UpdateEvent;
            type UpdateOutput = $UpdateOutput;

            fn run_query(&$self_, event: $QueryEvent) -> $QueryOutput {
                __lewis_impl_acidic_method_body! {
                    ($QueryEvent $QueryOutput)
                    event
                    ($($query_events)*)
                }
            }

            fn run_update(&mut $self_, event: $UpdateEvent) -> $UpdateOutput {
                __lewis_impl_acidic_method_body! {
                    ($UpdateEvent $UpdateOutput)
                    event
                    ($($update_events)*)
                }
            }
        }
    };
}

#[macro_export]
macro_rules! __lewis_impl_acidic_method_body {
    (
        ($Event:ident $Output:ident)
        $event:ident
        (
            $(
                ($method:ident ($($arg:ident: $Arg:ty),*) ($Out:ty) ($($body:tt)*))
            )*
        )
    ) => {
        match $event {
            $(
                $Event::$method($($arg),*) => $Output::$method({
                    $($body)*
                }),
            )*
        }
    };
}

#[macro_export]
macro_rules! __lewis_create_enums {
    (
        ($Event:ident $Output:ident)
        (
            $(
                ($method:ident ($($arg:ident: $Arg:ty),*) ($Out:ty) ($($body:tt)*))
            )*
        )
    ) => {
        #[derive(Serialize, Deserialize)]
        enum $Event {
            $(
                $method($($Arg),*)
            ),*
        }

        #[derive(Serialize, Deserialize)]
        enum $Output {
            $(
                $method($Out)
            ),*
        }
    };
}

#[macro_export]
macro_rules! __lewis_ext_trait {
    (
        $State:ident
        ($QueryEvent:ident $QueryOutput:ident $UpdateEvent:ident $UpdateOutput:ident)
        $AcidExt:ident
        $self_:ident
        ($($query_events:tt)*)
        ($($update_events:tt)*)
    ) => {
        trait $AcidExt {
            __lewis_ext_trait_fn_signatures! {
                $self_
                ($($query_events)*)
            }
            __lewis_ext_trait_fn_signatures! {
                $self_
                ($($update_events)*)
            }
        }

        impl $AcidExt for $crate::Acid<$State> {
            __lewis_ext_trait_fn_bodies! {
                $self_
                query
                ($QueryEvent $QueryOutput)
                ($($query_events)*)
            }
            __lewis_ext_trait_fn_bodies! {
                $self_
                update
                ($UpdateEvent $UpdateOutput)
                ($($update_events)*)
            }
        }
    };
}

#[macro_export]
macro_rules! __lewis_ext_trait_fn_signatures {
    (
        $self_:ident
        (
            $(
                ($method:ident ($($arg:ident: $Arg:ty),*) ($Out:ty) ($($body:tt)*))
            )*
        )
    ) => {
        $(
            fn $method(&$self_, $($arg: $Arg),*) -> $crate::Result<$Out>;
        )*
    };
}

#[macro_export]
macro_rules! __lewis_ext_trait_fn_bodies {
    // Special case: if there's only one variant, then we must omit the
    // catch-all pattern
    (
        $self_:ident
        $handle:ident
        ($Event:ident $Output:ident)
        (
            ($method:ident ($($arg:ident: $Arg:ty),*) ($Out:ty) ($($body:tt)*))
        )
    ) => {
        fn $method(&$self_, $($arg: $Arg),*) -> $crate::Result<$Out> {
            match $self_.$handle($Event::$method($($arg),*)) {
                Ok($Output::$method(r)) => Ok(r),
                // Ok(_) => unreachable!(),
                Err(e) => Err(e)
            }
        }
    };
    (
        $self_:ident
        $handle:ident
        ($Event:ident $Output:ident)
        (
            $(
                ($method:ident ($($arg:ident: $Arg:ty),*) ($Out:ty) ($($body:tt)*))
            )*
        )
    ) => {
        $(
            fn $method(&$self_, $($arg: $Arg),*) -> $crate::Result<$Out> {
                match $self_.$handle($Event::$method($($arg),*)) {
                    Ok($Output::$method(r)) => Ok(r),
                    Ok(_) => unreachable!(),
                    Err(e) => Err(e)
                }
            }
        )*
    };
}
