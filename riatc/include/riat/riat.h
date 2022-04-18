// For documentation, refer to riatc's documentation.

#ifndef RAT_IN_A_TUBE_H
#define RAT_IN_A_TUBE_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

typedef struct RIATCompiler RIATCompiler;
typedef struct RIATCompileError RIATCompileError;
typedef struct RIATCompiledScriptData RIATCompiledScriptData;

typedef enum RIATCompileTarget {
    RIAT_HaloCEA,
    RIAT_HaloCEXboxNTSC,
    RIAT_HaloCEGBX,
    RIAT_HaloCEGBXDemo,
    RIAT_HaloCustomEdition,
} RIATCompileTarget;

typedef struct RIATCompileErrorC {
    const char *file;
    const char *message;
    size_t line;
    size_t column;
    void *_reserved;
} RIATCompileErrorC;

typedef enum RIATCompileEncoding {
    RIAT_UTF8,
    RIAT_Windows1252
} RIATCompileEncoding;

RIATCompiler *riat_compiler_new(RIATCompileTarget target, RIATCompileEncoding encoding);
int riat_compiler_read_script_data(RIATCompiler *compiler, const char *input_filename, const uint8_t *input_data, size_t input_size, RIATCompileErrorC *error);
RIATCompiledScriptData *riat_compiler_compile_script_data(RIATCompiler *compiler, RIATCompileErrorC *error);
void riat_compiler_free(RIATCompiler *compiler);

size_t riat_script_data_get_warnings(const RIATCompiledScriptData *script_data, RIATCompileErrorC *warnings);
void riat_script_data_free(RIATCompiledScriptData *compiler);

void riat_error_free(RIATCompileErrorC *error);

typedef enum RIATValueType {
    RIAT_Unparsed,
    RIAT_SpecialForm,
    RIAT_FunctionName,
    RIAT_Passthrough,
    RIAT_Void,
    RIAT_Boolean,
    RIAT_Real,
    RIAT_Short,
    RIAT_Long,
    RIAT_String,
    RIAT_Script,
    RIAT_TriggerVolume,
    RIAT_CutsceneFlag,
    RIAT_CutsceneCameraPoint,
    RIAT_CutsceneTitle,
    RIAT_CutsceneRecording,
    RIAT_DeviceGroup,
    RIAT_Ai,
    RIAT_AiCommandList,
    RIAT_StartingProfile,
    RIAT_Conversation,
    RIAT_Navpoint,
    RIAT_HudMessage,
    RIAT_ObjectList,
    RIAT_Sound,
    RIAT_Effect,
    RIAT_Damage,
    RIAT_LoopingSound,
    RIAT_AnimationGraph,
    RIAT_ActorVariant,
    RIAT_DamageEffect,
    RIAT_ObjectDefinition,
    RIAT_GameDifficulty,
    RIAT_Team,
    RIAT_AiDefaultState,
    RIAT_ActorType,
    RIAT_HudCorner,
    RIAT_Object,
    RIAT_Unit,
    RIAT_Vehicle,
    RIAT_Weapon,
    RIAT_Device,
    RIAT_Scenery,
    RIAT_ObjectName,
    RIAT_UnitName,
    RIAT_VehicleName,
    RIAT_WeaponName,
    RIAT_DeviceName,
    RIAT_SceneryName
} RIATValueType;

typedef enum RIATScriptType {
    RIAT_Startup,
    RIAT_Dormant,
    RIAT_Continuous,
    RIAT_Static,
    RIAT_Stub
} RIATScriptType;

typedef enum RIATNodeTypeC {
    RIAT_Primitive,
    RIAT_Global,
    RIAT_FunctionCall,
    RIAT_ScriptCall
} RIATNodeTypeC;

typedef union RIATScriptNodeDataC {
    size_t offset;
    float real;
    int32_t long_int;
    int16_t short_int;
    bool boolean;
} RIATScriptNodeDataC;

typedef struct RIATScriptNodeC {
    const char *file;
    size_t line;
    size_t column;
    const char *string_data;
    uint16_t index_union;
    RIATValueType value_type;
    RIATNodeTypeC node_type;
    RIATScriptNodeDataC node_data;
    size_t next_node;
} RIATScriptNodeC;

size_t riat_script_data_get_nodes(const RIATCompiledScriptData *script_data, RIATScriptNodeC *nodes);

typedef struct RIATScriptC {
    const char *name;
    const char *file;
    size_t line;
    size_t column;
    RIATScriptType script_type;
    RIATValueType return_type;
    size_t first_node;
} RIATScriptC;

typedef struct RIATGlobalC {
    const char *name;
    const char *file;
    size_t line;
    size_t column;
    RIATValueType value_type;
    size_t first_node;
} RIATGlobalC;

size_t riat_script_data_get_scripts(const RIATCompiledScriptData *script_data, RIATScriptC *scripts);
size_t riat_script_data_get_globals(const RIATCompiledScriptData *script_data, RIATGlobalC *globals);

#ifdef __cplusplus
}
#endif

#endif
