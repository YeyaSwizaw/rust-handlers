//////////////////////////////////////////////////////////////////////////////
//  File: rust-handler/lib.rs
//////////////////////////////////////////////////////////////////////////////
//  Copyright 2016 Samuel Sleight
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
//////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! handlers {
    (
        $system_name:ident {
            $($handler_name:ident {
                $($signal_name:ident ( $($arg_name:ident : $arg_type:ty),* ) => $slot_name:ident);*
            })*
        }
    ) => {
        handlers! {
            DEFINE_HANDLERS $system_name

            $($handler_name {
                $($signal_name ( $($arg_name : $arg_type),* ) => $slot_name);*
            })*
        }

        interpolate_idents! {
            pub trait [$system_name Object] : $([As $handler_name] +)* {}
            pub trait [Is $system_name Object] {}

            impl<T> [$system_name Object] for T where T: $([As $handler_name] +)* {}

            #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
            pub struct [$system_name Index](usize);

            pub struct [$system_name] {
                objects: Vec<Box<[$system_name Object]>>,
                idxs: Vec<Option<usize>>,
                $([$handler_name _idxs]: Vec<usize>),*
            }

            impl [$system_name] {
                pub fn new() -> [$system_name] {
                    [$system_name] {
                        objects: Vec::new(),
                        idxs: Vec::new(),
                        $([$handler_name _idxs]: Vec::new()),*
                    }
                }

                pub fn add(&mut self, object: Box<[$system_name Object]>) -> [$system_name Index] {
                    let idx = self.idxs.len();
                    self.idxs.push(Some(self.objects.len()));
                    self.objects.push(object);
                    let object = self.objects.last().unwrap();
                    $(
                        if object.[as_ $handler_name]().is_some() {
                            println!("Added {}", stringify!($handler_name));
                            self.[$handler_name _idxs].push(idx);
                        };
                    )*
                    [$system_name Index](idx)
                }

                pub fn remove(&mut self, idx: [$system_name Index]) -> Option<Box<[$system_name Object]>> {
                    self.idxs.get(idx.0).cloned().and_then(
                        move |obj_idx: Option<usize>| obj_idx.map(
                            move |obj_idx: usize| unsafe {
                                let obj = self.objects.swap_remove(obj_idx);
                                *self.idxs.last_mut().unwrap() = Some(obj_idx);
                                *self.idxs.get_unchecked_mut(idx.0) = None;
                                obj
                            }
                        )
                    )
                }

                pub fn iter(&self) -> ::std::slice::Iter<Box<[$system_name Object]>> {
                    self.objects.iter()
                }

                $($(
                    pub fn [$signal_name](&mut self, $($arg_name : $arg_type,)*) {
                        unsafe {
                            let mut i = 0;
                            loop {
                                if i >= self.[$handler_name _idxs].len() {
                                    return
                                };

                                let idx = *self.[$handler_name _idxs].get_unchecked(i);
                                let idx = *self.idxs.get_unchecked(i);
                                if let Some(idx) = idx {
                                    self.objects.get_unchecked_mut(idx).[as_ $handler_name _mut]().unwrap().[$slot_name]($($arg_name),*);
                                    i += 1;
                                } else {
                                    self.[$handler_name _idxs].swap_remove(i);
                                }
                            }
                        }
                    }
                )*)*
            }
        }

    };

    (
        DEFINE_HANDLERS $system_name:ident

        $($handler_name:ident {
            $($signal_name:ident ( $($arg_name:ident : $arg_type:ty),* ) => $slot_name:ident);*
        })*
    ) => {
        $(
            pub trait $handler_name {
                $(fn $slot_name(&mut self, $($arg_name : $arg_type,)*);)*
            }

            interpolate_idents! {
                pub trait [As $handler_name] {
                    fn [as_ $handler_name](&self) -> Option<&$handler_name>;
                    fn [as_ $handler_name _mut](&mut self) -> Option<&mut $handler_name>;
                }

                impl<T> [As $handler_name] for T where T: [Is $system_name Object] + $handler_name {
                    fn [as_ $handler_name](&self) -> Option<&$handler_name> {
                        Some(self as &$handler_name)
                    }

                    fn [as_ $handler_name _mut](&mut self) -> Option<&mut $handler_name> {
                        Some(self as &mut $handler_name)
                    }
                }

                impl<T> [As $handler_name] for T where T: [Is $system_name Object] {
                    default fn [as_ $handler_name](&self) -> Option<&$handler_name> {
                        None
                    }

                    default fn [as_ $handler_name _mut](&mut self) -> Option<&mut $handler_name> {
                        None
                    }
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! handlers_objects {
    (
        $system_name:ident {
            $($object_name:ident),+
        }
    ) => (
        $(
            interpolate_idents! {
                impl [Is $system_name Object] for $object_name {}
            }
        )+
    )
}

