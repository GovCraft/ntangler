use lazy_static::lazy_static;
use termcolor::Color;
use owo_colors::{OwoColorize, Style};


pub(crate) const TAB_WIDTH: usize = 8; // You can set this to any number of spaces you want
pub(crate) const LIST_ROW: usize = 3;


lazy_static! {

// app colors
// Crimson
pub static ref ACCENT: Style = CRIMSON_9.clone();
pub static ref MAJOR: Style = CRIMSON_9.clone();
pub static ref MINOR: Style = CRIMSON_11.clone();
pub static ref PATCH: Style = CRIMSON_8.clone();

// Indigo
pub static ref INSTRUCTIONS: Style = INDIGO_11.clone();
pub static ref BRAND_NAME: Style = INDIGO_9.clone();

pub static ref DESCRIPTION_COLOR: Style = INDIGO_12.clone();
pub static ref TIME_COLOR: Style = INDIGO_12.clone();
pub static ref TIME_ELEMENTS_COLOR: Style = INDIGO_8.clone();
pub static ref HR_COLOR: Style = INDIGO_6.clone();

// Jade
pub static ref REPO_COLOR: Style = JADE_11.clone();
pub static ref COMMIT_TYPE_COLOR: Style = JADE_11.clone();
pub static ref SCOPE_COLOR: Style = JADE_12.clone();
pub static ref FILENAME: Style = JADE_9.clone();

// Gray

pub static ref STATUS: Style = YELLOW_9.clone();
// Lime
pub static ref TERTIARY_10: Style = LIME_10.clone();
// Yellow
pub static ref OID_COLOR: Style = YELLOW_12.clone();

pub static ref BG_DARK: Style = Style::new().truecolor(16, 18, 17);

pub static ref GRAY_1: Style = Style::new().truecolor(17, 17, 17);
pub static ref GRAY_2: Style = Style::new().truecolor(25, 25, 25);
pub static ref GRAY_3: Style = Style::new().truecolor(34, 34, 34);
pub static ref GRAY_4: Style = Style::new().truecolor(42, 42, 42);
pub static ref GRAY_5: Style = Style::new().truecolor(49, 49, 49);
pub static ref GRAY_6: Style = Style::new().truecolor(58, 58, 58);
pub static ref GRAY_7: Style = Style::new().truecolor(72, 72, 72);
pub static ref GRAY_8: Style = Style::new().truecolor(96, 96, 96);
pub static ref GRAY_9: Style = Style::new().truecolor(110, 110, 110);
pub static ref GRAY_10: Style = Style::new().truecolor(123, 123, 123);
pub static ref GRAY_11: Style = Style::new().truecolor(180, 180, 180);
pub static ref GRAY_12: Style = Style::new().truecolor(238, 238, 238);

pub static ref MAUVE_1: Style = Style::new().truecolor(18, 17, 19);
pub static ref MAUVE_2: Style = Style::new().truecolor(26, 25, 27);
pub static ref MAUVE_3: Style = Style::new().truecolor(35, 34, 37);
pub static ref MAUVE_4: Style = Style::new().truecolor(43, 41, 45);
pub static ref MAUVE_5: Style = Style::new().truecolor(50, 48, 53);
pub static ref MAUVE_6: Style = Style::new().truecolor(60, 57, 63);
pub static ref MAUVE_7: Style = Style::new().truecolor(73, 71, 78);
pub static ref MAUVE_8: Style = Style::new().truecolor(98, 95, 105);
pub static ref MAUVE_9: Style = Style::new().truecolor(111, 109, 120);
pub static ref MAUVE_10: Style = Style::new().truecolor(124, 122, 133);
pub static ref MAUVE_11: Style = Style::new().truecolor(181, 178, 188);
pub static ref MAUVE_12: Style = Style::new().truecolor(238, 238, 240);
}
lazy_static! {

pub static ref TOMATO_1: Style = Style::new().truecolor(24, 17, 17);
pub static ref TOMATO_2: Style = Style::new().truecolor(31, 21, 19);
pub static ref TOMATO_3: Style = Style::new().truecolor(57, 23, 20);
pub static ref TOMATO_4: Style = Style::new().truecolor(78, 21, 17);
pub static ref TOMATO_5: Style = Style::new().truecolor(94, 28, 22);
pub static ref TOMATO_6: Style = Style::new().truecolor(110, 41, 32);
pub static ref TOMATO_7: Style = Style::new().truecolor(133, 58, 45);
pub static ref TOMATO_8: Style = Style::new().truecolor(172, 77, 57);
pub static ref TOMATO_9: Style = Style::new().truecolor(229, 77, 46);
pub static ref TOMATO_10: Style = Style::new().truecolor(236, 97, 66);
pub static ref TOMATO_11: Style = Style::new().truecolor(255, 151, 125);
pub static ref TOMATO_12: Style = Style::new().truecolor(251, 211, 203);
}
lazy_static! {

    pub static ref RED_1: Style = Style::new().truecolor(25, 17, 17);
pub static ref RED_2: Style = Style::new().truecolor(32, 19, 20);
pub static ref RED_3: Style = Style::new().truecolor(59, 18, 25);
pub static ref RED_4: Style = Style::new().truecolor(80, 15, 28);
pub static ref RED_5: Style = Style::new().truecolor(97, 22, 35);
pub static ref RED_6: Style = Style::new().truecolor(114, 35, 45);
pub static ref RED_7: Style = Style::new().truecolor(140, 51, 58);
pub static ref RED_8: Style = Style::new().truecolor(181, 69, 72);
pub static ref RED_9: Style = Style::new().truecolor(229, 72, 77);
pub static ref RED_10: Style = Style::new().truecolor(236, 93, 94);
pub static ref RED_11: Style = Style::new().truecolor(255, 149, 146);
pub static ref RED_12: Style = Style::new().truecolor(255, 209, 217);

pub static ref RUBY_1: Style = Style::new().truecolor(25, 17, 19);
pub static ref RUBY_2: Style = Style::new().truecolor(30, 21, 23);
pub static ref RUBY_3: Style = Style::new().truecolor(58, 20, 30);
pub static ref RUBY_4: Style = Style::new().truecolor(78, 19, 37);
pub static ref RUBY_5: Style = Style::new().truecolor(94, 26, 46);
pub static ref RUBY_6: Style = Style::new().truecolor(111, 37, 57);
pub static ref RUBY_7: Style = Style::new().truecolor(136, 52, 71);
pub static ref RUBY_8: Style = Style::new().truecolor(179, 68, 90);
pub static ref RUBY_9: Style = Style::new().truecolor(229, 70, 102);
pub static ref RUBY_10: Style = Style::new().truecolor(236, 90, 114);
pub static ref RUBY_11: Style = Style::new().truecolor(255, 148, 157);
pub static ref RUBY_12: Style = Style::new().truecolor(254, 210, 225);

pub static ref CRIMSON_1: Style = Style::new().truecolor(25, 17, 20);
pub static ref CRIMSON_2: Style = Style::new().truecolor(32, 19, 24);
pub static ref CRIMSON_3: Style = Style::new().truecolor(56, 21, 37);
pub static ref CRIMSON_4: Style = Style::new().truecolor(77, 18, 47);
pub static ref CRIMSON_5: Style = Style::new().truecolor(92, 24, 57);
pub static ref CRIMSON_6: Style = Style::new().truecolor(109, 37, 69);
pub static ref CRIMSON_7: Style = Style::new().truecolor(135, 51, 86);
pub static ref CRIMSON_8: Style = Style::new().truecolor(176, 67, 110);
pub static ref CRIMSON_9: Style = Style::new().truecolor(233, 61, 130);
pub static ref CRIMSON_10: Style = Style::new().truecolor(238, 81, 138);
pub static ref CRIMSON_11: Style = Style::new().truecolor(255, 146, 173);
pub static ref CRIMSON_12: Style = Style::new().truecolor(253, 211, 232);

}
lazy_static! {

    pub static ref PINK_1: Style = Style::new().truecolor(25, 17, 23);
pub static ref PINK_2: Style = Style::new().truecolor(33, 18, 29);
pub static ref PINK_3: Style = Style::new().truecolor(55, 23, 47);
pub static ref PINK_4: Style = Style::new().truecolor(75, 20, 61);
pub static ref PINK_5: Style = Style::new().truecolor(89, 28, 71);
pub static ref PINK_6: Style = Style::new().truecolor(105, 41, 85);
pub static ref PINK_7: Style = Style::new().truecolor(131, 56, 105);
pub static ref PINK_8: Style = Style::new().truecolor(168, 72, 133);
pub static ref PINK_9: Style = Style::new().truecolor(214, 64, 159);
pub static ref PINK_10: Style = Style::new().truecolor(222, 81, 168);
pub static ref PINK_11: Style = Style::new().truecolor(255, 141, 204);
pub static ref PINK_12: Style = Style::new().truecolor(253, 209, 234);
pub static ref WHITE_PURE: Style = Style::new().truecolor(255, 255, 255);

pub static ref PLUM_1: Style = Style::new().truecolor(24, 17, 24);
pub static ref PLUM_2: Style = Style::new().truecolor(32, 19, 32);
pub static ref PLUM_3: Style = Style::new().truecolor(53, 26, 53);
pub static ref PLUM_4: Style = Style::new().truecolor(69, 29, 71);
pub static ref PLUM_5: Style = Style::new().truecolor(81, 36, 84);
pub static ref PLUM_6: Style = Style::new().truecolor(94, 48, 97);
pub static ref PLUM_7: Style = Style::new().truecolor(115, 64, 121);
pub static ref PLUM_8: Style = Style::new().truecolor(146, 84, 156);
pub static ref PLUM_9: Style = Style::new().truecolor(171, 74, 186);
pub static ref PLUM_10: Style = Style::new().truecolor(182, 88, 196);
pub static ref PLUM_11: Style = Style::new().truecolor(231, 150, 243);
pub static ref PLUM_12: Style = Style::new().truecolor(244, 212, 244);



}

lazy_static! {
    pub static ref PURPLE_1: Style = Style::new().truecolor(24, 17, 27);
pub static ref PURPLE_2: Style = Style::new().truecolor(30, 21, 35);
pub static ref PURPLE_3: Style = Style::new().truecolor(48, 28, 59);
pub static ref PURPLE_4: Style = Style::new().truecolor(61, 34, 78);
pub static ref PURPLE_5: Style = Style::new().truecolor(72, 41, 92);
pub static ref PURPLE_6: Style = Style::new().truecolor(84, 52, 107);
pub static ref PURPLE_7: Style = Style::new().truecolor(102, 66, 130);
pub static ref PURPLE_8: Style = Style::new().truecolor(132, 87, 170);
pub static ref PURPLE_9: Style = Style::new().truecolor(142, 78, 198);
pub static ref PURPLE_10: Style = Style::new().truecolor(154, 92, 208);
pub static ref PURPLE_11: Style = Style::new().truecolor(209, 157, 255);
pub static ref PURPLE_12: Style = Style::new().truecolor(236, 217, 250);

pub static ref VIOLET_1: Style = Style::new().truecolor(20, 18, 31);
pub static ref VIOLET_2: Style = Style::new().truecolor(27, 21, 37);
pub static ref VIOLET_3: Style = Style::new().truecolor(41, 31, 67);
pub static ref VIOLET_4: Style = Style::new().truecolor(51, 37, 91);
pub static ref VIOLET_5: Style = Style::new().truecolor(60, 46, 105);
pub static ref VIOLET_6: Style = Style::new().truecolor(71, 56, 118);
pub static ref VIOLET_7: Style = Style::new().truecolor(86, 70, 139);
pub static ref VIOLET_8: Style = Style::new().truecolor(105, 88, 173);
pub static ref VIOLET_9: Style = Style::new().truecolor(110, 86, 207);
pub static ref VIOLET_10: Style = Style::new().truecolor(125, 102, 217);
pub static ref VIOLET_11: Style = Style::new().truecolor(186, 167, 255);
pub static ref VIOLET_12: Style = Style::new().truecolor(226, 221, 254);
}

lazy_static! {
    pub static ref IRIS_1: Style = Style::new().truecolor(19, 19, 30);
pub static ref IRIS_2: Style = Style::new().truecolor(23, 22, 37);
pub static ref IRIS_3: Style = Style::new().truecolor(32, 34, 72);
pub static ref IRIS_4: Style = Style::new().truecolor(38, 42, 101);
pub static ref IRIS_5: Style = Style::new().truecolor(48, 51, 116);
pub static ref IRIS_6: Style = Style::new().truecolor(61, 62, 130);
pub static ref IRIS_7: Style = Style::new().truecolor(74, 74, 149);
pub static ref IRIS_8: Style = Style::new().truecolor(89, 88, 177);
pub static ref IRIS_9: Style = Style::new().truecolor(91, 91, 214);
pub static ref IRIS_10: Style = Style::new().truecolor(110, 106, 222);
pub static ref IRIS_11: Style = Style::new().truecolor(177, 169, 255);
pub static ref IRIS_12: Style = Style::new().truecolor(224, 223, 254);

pub static ref INDIGO_1: Style = Style::new().truecolor(17, 19, 31);
pub static ref INDIGO_2: Style = Style::new().truecolor(20, 23, 38);
pub static ref INDIGO_3: Style = Style::new().truecolor(24, 36, 73);
pub static ref INDIGO_4: Style = Style::new().truecolor(29, 46, 98);
pub static ref INDIGO_5: Style = Style::new().truecolor(37, 57, 116);
pub static ref INDIGO_6: Style = Style::new().truecolor(48, 67, 132);
pub static ref INDIGO_7: Style = Style::new().truecolor(58, 79, 151);
pub static ref INDIGO_8: Style = Style::new().truecolor(67, 93, 177);
pub static ref INDIGO_9: Style = Style::new().truecolor(62, 99, 221);
pub static ref INDIGO_10: Style = Style::new().truecolor(84, 114, 228);
pub static ref INDIGO_11: Style = Style::new().truecolor(158, 177, 255);
pub static ref INDIGO_12: Style = Style::new().truecolor(214, 225, 255);
}
lazy_static! {

pub static ref BLUE_1: Style = Style::new().truecolor(13, 21, 32);
pub static ref BLUE_2: Style = Style::new().truecolor(17, 25, 39);
pub static ref BLUE_3: Style = Style::new().truecolor(13, 40, 71);
pub static ref BLUE_4: Style = Style::new().truecolor(0, 51, 98);
pub static ref BLUE_5: Style = Style::new().truecolor(0, 64, 116);
pub static ref BLUE_6: Style = Style::new().truecolor(16, 77, 135);
pub static ref BLUE_7: Style = Style::new().truecolor(32, 93, 158);
pub static ref BLUE_8: Style = Style::new().truecolor(40, 112, 189);
pub static ref BLUE_9: Style = Style::new().truecolor(0, 144, 255);
pub static ref BLUE_10: Style = Style::new().truecolor(59, 158, 255);
pub static ref BLUE_11: Style = Style::new().truecolor(112, 184, 255);
pub static ref BLUE_12: Style = Style::new().truecolor(194, 230, 255);

pub static ref CYAN_1: Style = Style::new().truecolor(11, 22, 26);
pub static ref CYAN_2: Style = Style::new().truecolor(16, 27, 32);
pub static ref CYAN_3: Style = Style::new().truecolor(8, 44, 54);
pub static ref CYAN_4: Style = Style::new().truecolor(0, 56, 72);
pub static ref CYAN_5: Style = Style::new().truecolor(0, 69, 88);
pub static ref CYAN_6: Style = Style::new().truecolor(4, 84, 104);
pub static ref CYAN_7: Style = Style::new().truecolor(18, 103, 126);
pub static ref CYAN_8: Style = Style::new().truecolor(17, 128, 156);
pub static ref CYAN_9: Style = Style::new().truecolor(0, 162, 199);
pub static ref CYAN_10: Style = Style::new().truecolor(35, 175, 208);
pub static ref CYAN_11: Style = Style::new().truecolor(76, 204, 230);
pub static ref CYAN_12: Style = Style::new().truecolor(182, 236, 247);
}
lazy_static! {
    pub static ref TEAL_1: Style = Style::new().truecolor(13, 21, 20);
pub static ref TEAL_2: Style = Style::new().truecolor(17, 28, 27);
pub static ref TEAL_3: Style = Style::new().truecolor(13, 45, 42);
pub static ref TEAL_4: Style = Style::new().truecolor(2, 59, 55);
pub static ref TEAL_5: Style = Style::new().truecolor(8, 72, 67);
pub static ref TEAL_6: Style = Style::new().truecolor(20, 87, 80);
pub static ref TEAL_7: Style = Style::new().truecolor(28, 105, 97);
pub static ref TEAL_8: Style = Style::new().truecolor(32, 126, 115);
pub static ref TEAL_9: Style = Style::new().truecolor(18, 165, 148);
pub static ref TEAL_10: Style = Style::new().truecolor(14, 179, 158);
pub static ref TEAL_11: Style = Style::new().truecolor(11, 216, 182);
pub static ref TEAL_12: Style = Style::new().truecolor(173, 240, 221);

pub static ref JADE_1: Style = Style::new().truecolor(13, 21, 18);
pub static ref JADE_2: Style = Style::new().truecolor(18, 28, 24);
pub static ref JADE_3: Style = Style::new().truecolor(15, 46, 34);
pub static ref JADE_4: Style = Style::new().truecolor(11, 59, 44);
pub static ref JADE_5: Style = Style::new().truecolor(17, 72, 55);
pub static ref JADE_6: Style = Style::new().truecolor(27, 87, 69);
pub static ref JADE_7: Style = Style::new().truecolor(36, 104, 84);
pub static ref JADE_8: Style = Style::new().truecolor(42, 126, 104);
pub static ref JADE_9: Style = Style::new().truecolor(41, 163, 131);
pub static ref JADE_10: Style = Style::new().truecolor(39, 176, 139);
pub static ref JADE_11: Style = Style::new().truecolor(31, 216, 164);
pub static ref JADE_12: Style = Style::new().truecolor(173, 240, 212);

pub static ref GREEN_1: Style = Style::new().truecolor(14, 21, 18);
pub static ref GREEN_2: Style = Style::new().truecolor(18, 27, 23);
pub static ref GREEN_3: Style = Style::new().truecolor(19, 45, 33);
pub static ref GREEN_4: Style = Style::new().truecolor(17, 59, 41);
pub static ref GREEN_5: Style = Style::new().truecolor(23, 73, 51);
pub static ref GREEN_6: Style = Style::new().truecolor(32, 87, 62);
pub static ref GREEN_7: Style = Style::new().truecolor(40, 104, 74);
pub static ref GREEN_8: Style = Style::new().truecolor(47, 124, 87);
pub static ref GREEN_9: Style = Style::new().truecolor(48, 164, 108);
pub static ref GREEN_10: Style = Style::new().truecolor(51, 176, 116);
pub static ref GREEN_11: Style = Style::new().truecolor(61, 214, 140);
pub static ref GREEN_12: Style = Style::new().truecolor(177, 241, 203);
}
lazy_static! {

pub static ref GRASS_1: Style = Style::new().truecolor(14, 21, 17);
pub static ref GRASS_2: Style = Style::new().truecolor(20, 26, 21);
pub static ref GRASS_3: Style = Style::new().truecolor(27, 42, 30);
pub static ref GRASS_4: Style = Style::new().truecolor(29, 58, 36);
pub static ref GRASS_5: Style = Style::new().truecolor(37, 72, 45);
pub static ref GRASS_6: Style = Style::new().truecolor(45, 87, 54);
pub static ref GRASS_7: Style = Style::new().truecolor(54, 103, 64);
pub static ref GRASS_8: Style = Style::new().truecolor(62, 121, 73);
pub static ref GRASS_9: Style = Style::new().truecolor(70, 167, 88);
pub static ref GRASS_10: Style = Style::new().truecolor(83, 179, 101);
pub static ref GRASS_11: Style = Style::new().truecolor(113, 208, 131);
pub static ref GRASS_12: Style = Style::new().truecolor(194, 240, 194);

pub static ref BROWN_1: Style = Style::new().truecolor(18, 17, 15);
pub static ref BROWN_2: Style = Style::new().truecolor(28, 24, 22);
pub static ref BROWN_3: Style = Style::new().truecolor(40, 33, 29);
pub static ref BROWN_4: Style = Style::new().truecolor(50, 41, 34);
pub static ref BROWN_5: Style = Style::new().truecolor(62, 49, 40);
pub static ref BROWN_6: Style = Style::new().truecolor(77, 60, 47);
pub static ref BROWN_7: Style = Style::new().truecolor(97, 74, 57);
pub static ref BROWN_8: Style = Style::new().truecolor(124, 95, 70);
pub static ref BROWN_9: Style = Style::new().truecolor(173, 127, 88);
pub static ref BROWN_10: Style = Style::new().truecolor(184, 140, 103);
pub static ref BROWN_11: Style = Style::new().truecolor(219, 181, 148);
pub static ref BROWN_12: Style = Style::new().truecolor(242, 225, 202);
}

lazy_static! {

pub static ref BRONZE_1: Style = Style::new().truecolor(20, 17, 16);
pub static ref BRONZE_2: Style = Style::new().truecolor(28, 25, 23);
pub static ref BRONZE_3: Style = Style::new().truecolor(38, 34, 32);
pub static ref BRONZE_4: Style = Style::new().truecolor(48, 42, 39);
pub static ref BRONZE_5: Style = Style::new().truecolor(59, 51, 48);
pub static ref BRONZE_6: Style = Style::new().truecolor(73, 62, 58);
pub static ref BRONZE_7: Style = Style::new().truecolor(90, 76, 71);
pub static ref BRONZE_8: Style = Style::new().truecolor(111, 95, 88);
pub static ref BRONZE_9: Style = Style::new().truecolor(161, 128, 114);
pub static ref BRONZE_10: Style = Style::new().truecolor(174, 140, 126);
pub static ref BRONZE_11: Style = Style::new().truecolor(212, 179, 165);
pub static ref BRONZE_12: Style = Style::new().truecolor(237, 224, 217);

}
lazy_static! {
pub static ref GOLD_1: Style = Style::new().truecolor(18, 18, 17);
pub static ref GOLD_2: Style = Style::new().truecolor(27, 26, 23);
pub static ref GOLD_3: Style = Style::new().truecolor(36, 35, 31);
pub static ref GOLD_4: Style = Style::new().truecolor(45, 43, 38);
pub static ref GOLD_5: Style = Style::new().truecolor(56, 53, 46);
pub static ref GOLD_6: Style = Style::new().truecolor(68, 64, 57);
pub static ref GOLD_7: Style = Style::new().truecolor(84, 79, 70);
pub static ref GOLD_8: Style = Style::new().truecolor(105, 98, 86);
pub static ref GOLD_9: Style = Style::new().truecolor(151, 131, 101);
pub static ref GOLD_10: Style = Style::new().truecolor(163, 144, 115);
pub static ref GOLD_11: Style = Style::new().truecolor(203, 185, 159);
pub static ref GOLD_12: Style = Style::new().truecolor(232, 226, 217);

pub static ref SKY_1: Style = Style::new().truecolor(13, 20, 31);
pub static ref SKY_2: Style = Style::new().truecolor(17, 26, 39);
pub static ref SKY_3: Style = Style::new().truecolor(17, 40, 64);
pub static ref SKY_4: Style = Style::new().truecolor(17, 53, 85);
pub static ref SKY_5: Style = Style::new().truecolor(21, 68, 103);
pub static ref SKY_6: Style = Style::new().truecolor(27, 83, 123);
pub static ref SKY_7: Style = Style::new().truecolor(31, 102, 146);
pub static ref SKY_8: Style = Style::new().truecolor(25, 124, 174);
pub static ref SKY_9: Style = Style::new().truecolor(124, 226, 254);
pub static ref SKY_10: Style = Style::new().truecolor(168, 238, 255);
pub static ref SKY_11: Style = Style::new().truecolor(117, 199, 240);
pub static ref SKY_12: Style = Style::new().truecolor(194, 243, 255);
}
lazy_static! {

pub static ref MINT_1: Style = Style::new().truecolor(14, 21, 21);
pub static ref MINT_2: Style = Style::new().truecolor(15, 27, 27);
pub static ref MINT_3: Style = Style::new().truecolor(9, 44, 43);
pub static ref MINT_4: Style = Style::new().truecolor(0, 58, 56);
pub static ref MINT_5: Style = Style::new().truecolor(0, 71, 68);
pub static ref MINT_6: Style = Style::new().truecolor(16, 87, 80);
pub static ref MINT_7: Style = Style::new().truecolor(30, 104, 95);
pub static ref MINT_8: Style = Style::new().truecolor(39, 127, 112);
pub static ref MINT_9: Style = Style::new().truecolor(134, 234, 212);
pub static ref MINT_10: Style = Style::new().truecolor(168, 245, 229);
pub static ref MINT_11: Style = Style::new().truecolor(88, 213, 186);
pub static ref MINT_12: Style = Style::new().truecolor(196, 245, 225);

pub static ref LIME_1: Style = Style::new().truecolor(17, 19, 12);
pub static ref LIME_2: Style = Style::new().truecolor(21, 26, 16);
pub static ref LIME_3: Style = Style::new().truecolor(31, 41, 23);
pub static ref LIME_4: Style = Style::new().truecolor(41, 55, 29);
pub static ref LIME_5: Style = Style::new().truecolor(51, 68, 35);
pub static ref LIME_6: Style = Style::new().truecolor(61, 82, 42);
pub static ref LIME_7: Style = Style::new().truecolor(73, 98, 49);
pub static ref LIME_8: Style = Style::new().truecolor(87, 117, 56);
pub static ref LIME_9: Style = Style::new().truecolor(189, 238, 99);
pub static ref LIME_10: Style = Style::new().truecolor(212, 255, 112);
pub static ref LIME_11: Style = Style::new().truecolor(189, 229, 108);
pub static ref LIME_12: Style = Style::new().truecolor(227, 247, 186);
}
lazy_static! {

pub static ref YELLOW_1: Style = Style::new().truecolor(20, 18, 11);
pub static ref YELLOW_2: Style = Style::new().truecolor(27, 24, 15);
pub static ref YELLOW_3: Style = Style::new().truecolor(45, 35, 5);
pub static ref YELLOW_4: Style = Style::new().truecolor(54, 43, 0);
pub static ref YELLOW_5: Style = Style::new().truecolor(67, 53, 0);
pub static ref YELLOW_6: Style = Style::new().truecolor(82, 66, 2);
pub static ref YELLOW_7: Style = Style::new().truecolor(102, 84, 23);
pub static ref YELLOW_8: Style = Style::new().truecolor(131, 106, 33);
pub static ref YELLOW_9: Style = Style::new().truecolor(255, 230, 41);
pub static ref YELLOW_10: Style = Style::new().truecolor(255, 255, 87);
pub static ref YELLOW_11: Style = Style::new().truecolor(245, 225, 71);
pub static ref YELLOW_12: Style = Style::new().truecolor(246, 238, 180);

pub static ref AMBER_1: Style = Style::new().truecolor(22, 18, 12);
pub static ref AMBER_2: Style = Style::new().truecolor(29, 24, 15);
pub static ref AMBER_3: Style = Style::new().truecolor(48, 32, 8);
pub static ref AMBER_4: Style = Style::new().truecolor(63, 39, 0);
pub static ref AMBER_5: Style = Style::new().truecolor(77, 48, 0);
pub static ref AMBER_6: Style = Style::new().truecolor(92, 61, 5);
pub static ref AMBER_7: Style = Style::new().truecolor(113, 79, 25);
pub static ref AMBER_8: Style = Style::new().truecolor(143, 100, 36);
pub static ref AMBER_9: Style = Style::new().truecolor(255, 197, 61);
pub static ref AMBER_10: Style = Style::new().truecolor(255, 214, 10);
pub static ref AMBER_11: Style = Style::new().truecolor(255, 202, 22);
pub static ref AMBER_12: Style = Style::new().truecolor(255, 231, 179);
}
lazy_static! {

pub static ref ORANGE_1: Style = Style::new().truecolor(23, 18, 14);
pub static ref ORANGE_2: Style = Style::new().truecolor(30, 22, 15);
pub static ref ORANGE_3: Style = Style::new().truecolor(51, 30, 11);
pub static ref ORANGE_4: Style = Style::new().truecolor(70, 33, 0);
pub static ref ORANGE_5: Style = Style::new().truecolor(86, 40, 0);
pub static ref ORANGE_6: Style = Style::new().truecolor(102, 53, 12);
pub static ref ORANGE_7: Style = Style::new().truecolor(126, 69, 29);
pub static ref ORANGE_8: Style = Style::new().truecolor(163, 88, 41);
pub static ref ORANGE_9: Style = Style::new().truecolor(247, 107, 21);
pub static ref ORANGE_10: Style = Style::new().truecolor(255, 128, 31);
pub static ref ORANGE_11: Style = Style::new().truecolor(255, 160, 87);
pub static ref ORANGE_12: Style = Style::new().truecolor(255, 224, 194);


    }