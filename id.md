### ID

(000..100) => TransType::Tool(ToolType::Wrench),
(100..200) => TransType::Tool(ToolType::Solder),
(200..300) => TransType::Tool(ToolType::Crowbar),
(300..400) => TransType::Tool(ToolType::Screwdriver),
(400..500) => TransType::Tool(ToolType::WireNipper),
(600..700) => TransType::Tool(ToolType::SoftHammer),
(2000..4000) => TransType::Item,
(4000..6000) => TransType::Fluid,
_ => TransType::Trolley,
