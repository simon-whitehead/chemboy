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

macro_rules! guard {
    ( $x:expr ) => {
        if !$x {
            return;
        }
    }
}