use gpui::{Pixels, px};

pub struct Spacing;
impl Spacing {
    pub fn px_0() -> Pixels {
        px(0.)
    }
    pub fn px_0_5() -> Pixels {
        px(2.)
    }
    pub fn px_1() -> Pixels {
        px(4.)
    }
    pub fn px_1_5() -> Pixels {
        px(6.)
    }
    pub fn px_2() -> Pixels {
        px(8.)
    }
    pub fn px_2_5() -> Pixels {
        px(10.)
    }
    pub fn px_3() -> Pixels {
        px(12.)
    }
    pub fn px_4() -> Pixels {
        px(16.)
    }
    pub fn px_5() -> Pixels {
        px(20.)
    }
    pub fn px_6() -> Pixels {
        px(24.)
    }
}

pub struct Sizes;
impl Sizes {
    pub fn xs() -> Pixels {
        px(20.)
    }
    pub fn sm() -> Pixels {
        px(24.)
    }
    pub fn md() -> Pixels {
        px(28.)
    }
    pub fn lg() -> Pixels {
        px(32.)
    }
    pub fn icon_xs() -> Pixels {
        px(16.)
    }
    pub fn icon_sm() -> Pixels {
        px(20.)
    }
    pub fn icon_md() -> Pixels {
        px(24.)
    }
}
