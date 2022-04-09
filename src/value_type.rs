/// Value type, used for typing for parameters, return types, and globals
#[derive(Copy, Clone)]
pub enum ValueType {
    Unparsed,
    SpecialForm,
    FunctionName,
    Passthrough,
    Void,
    Boolean,
    Real,
    Short,
    Long,
    String,
    Script,
    TriggerVolume,
    CutsceneFlag,
    CutsceneCameraPoint,
    CutsceneTitle,
    CutsceneRecording,
    DeviceGroup,
    Ai,
    AiCommandList,
    StartingProfile,
    Conversation,
    Navpoint,
    HudMessage,
    ObjectList,
    Sound,
    Effect,
    Damage,
    LoopingSound,
    AnimationGraph,
    ActorVariant,
    DamageEffect,
    ObjectDefinition,
    GameDifficulty,
    Team,
    AiDefaultState,
    ActorType,
    HudCorner,
    Object,
    Unit,
    Vehicle,
    Weapon,
    Device,
    Scenery,
    ObjectName,
    UnitName,
    VehicleName,
    WeaponName,
    DeviceName,
    SceneryName
}

impl ToString for ValueType {
    fn to_string(&self) -> String {
        self.as_str().to_owned()
    }
}

impl ValueType {
    pub fn as_str(&self) -> &str {
        match *self {
            ValueType::Unparsed => "unparsed",
            ValueType::SpecialForm => "special form",
            ValueType::FunctionName => "function name",
            ValueType::Passthrough => "passthrough",
            ValueType::Void => "void",
            ValueType::Boolean => "boolean",
            ValueType::Real => "real",
            ValueType::Short => "short",
            ValueType::Long => "long",
            ValueType::String => "string",
            ValueType::Script => "script",
            ValueType::TriggerVolume => "trigger volume",
            ValueType::CutsceneFlag => "cutscene flag",
            ValueType::CutsceneCameraPoint => "cutscene camera point",
            ValueType::CutsceneTitle => "cutscene title",
            ValueType::CutsceneRecording => "cutscene recording",
            ValueType::DeviceGroup => "device group",
            ValueType::Ai => "ai",
            ValueType::AiCommandList => "ai command list",
            ValueType::StartingProfile => "starting profile",
            ValueType::Conversation => "conversation",
            ValueType::Navpoint => "navpoint",
            ValueType::HudMessage => "hud message",
            ValueType::ObjectList => "object list",
            ValueType::Sound => "sound",
            ValueType::Effect => "effect",
            ValueType::Damage => "damage",
            ValueType::LoopingSound => "looping sound",
            ValueType::AnimationGraph => "animation graph",
            ValueType::ActorVariant => "actor variant",
            ValueType::DamageEffect => "damage effect",
            ValueType::ObjectDefinition => "object definition",
            ValueType::GameDifficulty => "game difficulty",
            ValueType::Team => "team",
            ValueType::AiDefaultState => "ai default state",
            ValueType::ActorType => "actor type",
            ValueType::HudCorner => "hud corner",
            ValueType::Object => "object",
            ValueType::Unit => "unit",
            ValueType::Vehicle => "vehicle",
            ValueType::Weapon => "weapon",
            ValueType::Device => "device",
            ValueType::Scenery => "scenery",
            ValueType::ObjectName => "object name",
            ValueType::UnitName => "unit name",
            ValueType::VehicleName => "vehicle name",
            ValueType::WeaponName => "weapon name",
            ValueType::DeviceName => "device name",
            ValueType::SceneryName => "scenery name"
        }
    }

    pub fn as_int(&self) -> u16 {
        *self as u16
    }
}
