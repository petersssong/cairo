use crate::box::BoxTrait;

mod enum_value {
    pub extern type Const<T, const VARIANT_INDEX: felt252, V>;
}
mod value {
    pub extern type Const<T, const VALUE: felt252>;
}
mod struct0 {
    pub extern type Const<T>;
}
mod struct2 {
    pub extern type Const<T, C0, C1>;
}

mod const_bool {
    pub extern fn const_as_box<T, const SEGMENT_ID: felt252>() -> Box<bool> nopanic;
}

#[test]
fn test_const_bool() {
    assert!(
        !const_bool::const_as_box::<enum_value::Const<bool, 0, struct0::Const<()>>, 0>().unbox(),
    );
    assert!(
        const_bool::const_as_box::<enum_value::Const<bool, 1, struct0::Const<()>>, 0>().unbox(),
    );
}

mod const_felt252 {
    pub extern fn const_as_box<T, const SEGMENT_ID: felt252>() -> Box<felt252> nopanic;
}

#[test]
fn test_const_felt252() {
    assert!(const_felt252::const_as_box::<value::Const<felt252, 0>, 1>().unbox() == 0);
    assert!(const_felt252::const_as_box::<value::Const<felt252, -1>, 2>().unbox() == -1);
}

mod const_u8 {
    pub extern fn const_as_box<T, const SEGMENT_ID: felt252>() -> Box<u8> nopanic;
}

#[test]
fn test_const_u8() {
    assert!(const_u8::const_as_box::<value::Const<u8, 0>, 0>().unbox() == 0);
    assert!(const_u8::const_as_box::<value::Const<u8, 255>, 1>().unbox() == 255);
}

mod const_u256 {
    pub extern fn const_as_box<T, const SEGMENT_ID: felt252>() -> Box<u256> nopanic;
}

#[test]
fn test_const_u256() {
    assert!(
        const_u256::const_as_box::<
            struct2::Const<u256, value::Const<u128, 0x10>, value::Const<u128, 0>>, 0,
        >()
            .unbox() == 0x10,
    );
    assert!(
        const_u256::const_as_box::<
            struct2::Const<u256, value::Const<u128, 0>, value::Const<u128, 0x10>>, 0,
        >()
            .unbox() == 0x1000000000000000000000000000000000,
    );
}

#[derive(Copy, Drop, PartialEq, Debug)]
enum ThreeOptions {
    A: felt252,
    B: u256,
    C,
}

mod const_three_options {
    pub extern fn const_as_box<T, const SEGMENT_ID: felt252>() -> Box<super::ThreeOptions> nopanic;
}

#[test]
fn test_complex_enum() {
    assert!(
        const_three_options::const_as_box::<
            enum_value::Const<ThreeOptions, 0, value::Const<felt252, -1>>, 0,
        >()
            .unbox() == ThreeOptions::A(-1),
    );
    assert!(
        const_three_options::const_as_box::<
            enum_value::Const<
                ThreeOptions,
                1,
                struct2::Const<u256, value::Const<u128, 0x10>, value::Const<u128, 0x20>>,
            >,
            1,
        >()
            .unbox() == ThreeOptions::B(0x2000000000000000000000000000000010),
    );
    assert!(
        const_three_options::const_as_box::<
            enum_value::Const<ThreeOptions, 2, struct0::Const<()>>, 1,
        >()
            .unbox() == ThreeOptions::C,
    );
}

#[derive(Copy, Drop, PartialEq, Debug)]
enum ThreeOptions2 {
    A: felt252,
    B: (u256, u256),
    C,
}

mod const_tuple_three_options {
    pub extern fn const_as_box<T, const SEGMENT_INDEX: felt252>() -> Box<
        (super::ThreeOptions2, super::ThreeOptions2),
    > nopanic;
}

#[test]
fn test_two_complex_enums() {
    assert!(
        const_tuple_three_options::const_as_box::<
            struct2::Const<
                (ThreeOptions2, ThreeOptions2),
                enum_value::Const<ThreeOptions2, 0, value::Const<felt252, 1337>>,
                enum_value::Const<ThreeOptions2, 2, struct0::Const<()>>,
            >,
            0,
        >()
            .unbox() == (ThreeOptions2::A(1337), ThreeOptions2::C),
    );
}

#[test]
fn test_complex_consts() {
    const VAR_AND_MATCH_CONST: felt252 = {
        let x = Some((1, 2_u8));
        match x {
            Some((v, _)) => v,
            None => 3,
        }
    };
    assert_eq!(VAR_AND_MATCH_CONST, 1);
    const TRUE: bool = true;
    const IF_CONST_TRUE: felt252 = if TRUE {
        4
    } else {
        5
    };
    assert_eq!(IF_CONST_TRUE, 4);
    const FALSE: bool = false;
    const IF_CONST_FALSE: felt252 = if FALSE {
        6
    } else {
        7
    };
    assert_eq!(IF_CONST_FALSE, 7);
}

mod const_starknet_consts {
    pub extern fn const_as_box<T, const SEGMENT_INDEX: felt252>() -> Box<
        (starknet::ContractAddress, starknet::ClassHash),
    > nopanic;
}

const STARKNET_CONSTS: (starknet::ContractAddress, starknet::ClassHash) = (
    1000.try_into().unwrap(), 1001.try_into().unwrap(),
);

#[test]
fn test_starknet_consts() {
    assert!(
        const_starknet_consts::const_as_box::<
            struct2::Const<
                (starknet::ContractAddress, starknet::ClassHash),
                value::Const<starknet::ContractAddress, 1000>,
                value::Const<starknet::ClassHash, 1001>,
            >,
            0,
        >()
            .unbox() == STARKNET_CONSTS,
    );
}

