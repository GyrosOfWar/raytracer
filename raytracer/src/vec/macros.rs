#[macro_export]
macro_rules! impl_binary_op {
    ($op:tt : $method:ident => (
           $lhs_i:ident : $lhs_t:path,
           $rhs_i:ident : $rhs_t:path
        ) -> $return_t:path $body:block
    ) => {
        impl std::ops::$op<$rhs_t> for $lhs_t {
            type Output = $return_t;

            fn $method(self, $rhs_i: $rhs_t) -> $return_t {
                let $lhs_i = self;
                $body
            }
        }
        impl std::ops::$op<&$rhs_t> for $lhs_t {
            type Output = $return_t;

            fn $method(self, $rhs_i: &$rhs_t) -> $return_t {
                let $lhs_i = self;
                $body
            }
        }
        impl std::ops::$op<$rhs_t> for &$lhs_t {
            type Output = $return_t;

            fn $method(self, $rhs_i: $rhs_t) -> $return_t {
                let $lhs_i = self;
                $body
            }
        }
        impl std::ops::$op<&$rhs_t> for &$lhs_t {
            type Output = $return_t;

            fn $method(self, $rhs_i: &$rhs_t) -> $return_t {
                let $lhs_i = self;
                $body
            }
        }
    };
}
