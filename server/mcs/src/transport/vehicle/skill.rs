use crate::constant;

#[derive(Debug, PartialEq, Clone, sqlx::Type)]
#[sqlx(type_name = "tooltype")]
pub enum ToolType {
    #[sqlx(rename = "wrench")]
    Wrench, // 扳手
    #[sqlx(rename = "solder")]
    Solder, // 烙铁
    #[sqlx(rename = "crowbar")]
    Crowbar, // 撬棍
    #[sqlx(rename = "screwdriver")]
    Screwdriver, // 螺丝刀
    #[sqlx(rename = "wire_nipper")]
    WireNipper, // 剪线钳
    #[sqlx(rename = "soft_hammer")]
    SoftHammer, // 软锤
}

#[derive(Debug, PartialEq)]
pub enum Skill {
    Item,
    Fluid,
    UseTool(ToolType),
}

impl Skill {
    pub fn from_id(id: &i32) -> Self {
        if constant::ITEM_VEHICLE_ID_RANGE.contains(id) {
            return Self::Item;
        }
        if constant::FLUID_VEHICLE_ID_RANGE.contains(id) {
            return Self::Fluid;
        }
        if constant::USE_TOOL_WRENCH_VEHICLE_ID_RANGE.contains(id) {
            return Self::UseTool(ToolType::Wrench);
        }
        if constant::USE_TOOL_SOLDER_VEHICLE_ID_RANGE.contains(id) {
            return Self::UseTool(ToolType::Solder);
        }
        if constant::USE_TOOL_CROWBAR_VEHICLE_ID_RANGE.contains(id) {
            return Self::UseTool(ToolType::Crowbar);
        }
        if constant::USE_TOOL_SCREWDRIVER_VEHICLE_ID_RANGE.contains(id) {
            return Self::UseTool(ToolType::Screwdriver);
        }
        if constant::USE_TOOL_WIRENIPPER_VEHICLE_ID_RANGE.contains(id) {
            return Self::UseTool(ToolType::WireNipper);
        }
        if constant::USE_TOOL_SOFT_HAMMER_VEHICLE_ID_RANGE.contains(id) {
            return Self::UseTool(ToolType::SoftHammer);
        }
        panic!("id: {}, can not have any skill.", id);
    }
}
