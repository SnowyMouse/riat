/// Value type, used for typing for parameters, return types, and globals
#[derive(Copy, Clone, PartialEq, Debug)]
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

impl Default for ValueType {
    fn default() -> ValueType {
        ValueType::Unparsed
    }
}

impl ToString for ValueType {
    fn to_string(&self) -> String {
        self.as_str().to_owned()
    }
}

impl ValueType {
    pub fn can_convert_to(&self, to: ValueType) -> bool {
        match *self {
            // Anything matches itself
            n if n == to => true,

            // Anything can convert into a void or passthrough
            _ if to == ValueType::Void => true,


            // Passthrough can become anything else
            ValueType::Passthrough => true,


            // Reals can convert into any integer type
            ValueType::Real => to == ValueType::Long || to == ValueType::Short,

            // Shorts can ONLY convert into reals but NOT longs. This is probably a bug in Halo
            ValueType::Short => to == ValueType::Real,

            // Longs can be demoted into shorts or converted into a real number
            ValueType::Long => to == ValueType::Short || to == ValueType::Real,


            // Vehicles can be converted into units
            ValueType::Vehicle if to == ValueType::Unit => true,


            // Objects can be converted into object lists and objects
            ValueType::ObjectName | ValueType::Object | ValueType::Unit | ValueType::Weapon | ValueType::Scenery | ValueType::Vehicle | ValueType::Device => to == ValueType::Object || to == ValueType::ObjectList,


            // Anything not covered is false
            _ => false
        }
    }

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

    pub fn from_str_underscore(string: &str) -> Option<ValueType> {
        Some(match string {
            "unparsed" => ValueType::Unparsed,
            "special_form" => ValueType::SpecialForm,
            "function_name" => ValueType::FunctionName,
            "passthrough" => ValueType::Passthrough,
            "void" => ValueType::Void,
            "boolean" => ValueType::Boolean,
            "real" => ValueType::Real,
            "short" => ValueType::Short,
            "long" => ValueType::Long,
            "string" => ValueType::String,
            "script" => ValueType::Script,
            "trigger_volume" => ValueType::TriggerVolume,
            "cutscene_flag" => ValueType::CutsceneFlag,
            "cutscene_camera_point" => ValueType::CutsceneCameraPoint,
            "cutscene_title" => ValueType::CutsceneTitle,
            "cutscene_recording" => ValueType::CutsceneRecording,
            "device_group" => ValueType::DeviceGroup,
            "ai" => ValueType::Ai,
            "ai_command_list" => ValueType::AiCommandList,
            "starting_profile" => ValueType::StartingProfile,
            "conversation" => ValueType::Conversation,
            "navpoint" => ValueType::Navpoint,
            "hud_message" => ValueType::HudMessage,
            "object_list" => ValueType::ObjectList,
            "sound" => ValueType::Sound,
            "effect" => ValueType::Effect,
            "damage" => ValueType::Damage,
            "looping_sound" => ValueType::LoopingSound,
            "animation_graph" => ValueType::AnimationGraph,
            "actor_variant" => ValueType::ActorVariant,
            "damage_effect" => ValueType::DamageEffect,
            "object_definition" => ValueType::ObjectDefinition,
            "game_difficulty" => ValueType::GameDifficulty,
            "team" => ValueType::Team,
            "ai_default_state" => ValueType::AiDefaultState,
            "actor_type" => ValueType::ActorType,
            "hud_corner" => ValueType::HudCorner,
            "object" => ValueType::Object,
            "unit" => ValueType::Unit,
            "vehicle" => ValueType::Vehicle,
            "weapon" => ValueType::Weapon,
            "device" => ValueType::Device,
            "scenery" => ValueType::Scenery,
            "object_name" => ValueType::ObjectName,
            "unit_name" => ValueType::UnitName,
            "vehicle_name" => ValueType::VehicleName,
            "weapon_name" => ValueType::WeaponName,
            "device_name" => ValueType::DeviceName,
            "scenery_name" => ValueType::SceneryName,

            _ => return None
        })
    }

    pub fn as_int(&self) -> u16 {
        *self as u16
    }
}
