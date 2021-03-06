#[macro_export]
macro_rules! gb_asm {
    ( $( $x:expr )* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}

macro_rules! requires {
    ( $x:expr ) => {
        if !$x {
            return;
        }
    }
}

macro_rules! variant_equals {
    ( $variant:pat, $val:expr ) => {
        if let $variant = $val { true } else { false }
    }
}