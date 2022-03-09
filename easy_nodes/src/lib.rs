#[derive(Debug, Clone)]
pub struct Node<C, T> {
    common: C,
    data: Box<T>,
}

impl<C, T> Node<C, T> {
    pub fn new(common: C, data: T) -> Self {
        Self {
            common,
            data: Box::new(data),
        }
    }

    pub fn get_common(&self) -> &C {
        &self.common
    }

    pub fn get_data(&self) -> &T {
        &self.data
    }

    pub fn get_common_mut(&mut self) -> &mut C {
        &mut self.common
    }

    pub fn get_data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

#[macro_export]
macro_rules! node_system {
    (
        pub trait $visitor:ident<$common_type:ty> {
            $(fn $v_method:ident<$data_type:ty>();)*
        }

        $(
            #[consumer = $nv_method:ident()]
            $n_vis:vis $n_type:ident $node:ident $($node_tt:tt)?
        )*
    ) => {
        pub trait $visitor<T: Default> {
            $(
                fn $v_method(&mut self, common: &$common_type, data: &$data_type) -> T {
                    Default::default()
                }
            )*
        }

        pub mod _node_traits {
            pub trait $visitor {
                fn accept<T: Default>(&self, visitor: &mut dyn super::$visitor<T>) -> T;
            }
        }
        use _node_traits::$visitor as __node_traits;

        $(
            #[derive(Clone)]
            $n_vis $n_type $node $($node_tt)?

            impl $node {
                $n_vis fn to_node(self, common: $common_type) -> $crate::Node<$common_type, $node> {
                    $crate::Node::new(common, self)
                }
            }

            impl _node_traits::$visitor for $crate::Node<$common_type, $node> {
                fn accept<T: Default>(&self, visitor: &mut dyn $visitor<T>) -> T {
                    visitor.$nv_method(self.get_common(), self.get_data())
                }
            }
        )*
    }
}
