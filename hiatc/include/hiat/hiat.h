// For documentation, refer to hiatc's documentation.

#ifndef HAMSTER_IN_A_TUBE_H
#define HAMSTER_IN_A_TUBE_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

typedef struct HIATCompiler HIATCompiler;
typedef struct HIATCompileError HIATCompileError;
typedef struct HIATCompiledScriptData HIATCompiledScriptData;

typedef enum HIATCompileTarget {
    HIAT_HaloCEA,
    HIAT_HaloCEXboxNTSC,
    HIAT_HaloCEGBX,
    HIAT_HaloCEGBXDemo,
    HIAT_HaloCustomEdition,
} HIATCompileTarget;

typedef struct HIATCompileErrorC {
    const char *file;
    const char *message;
    size_t line;
    size_t column;
    void *_reserved;
} HIATCompileErrorC;

typedef enum HIATCompileEncoding {
    HIAT_UTF8,
    HIAT_Windows1252
} HIATCompileEncoding;

HIATCompiler *hiat_compiler_new(HIATCompileTarget target, HIATCompileEncoding encoding);
int hiat_compiler_read_script_data(HIATCompiler *compiler, const char *input_filename, const uint8_t *input_data, size_t input_size, HIATCompileErrorC *error);
HIATCompiledScriptData *hiat_compiler_compile_script_data(HIATCompiler *compiler, HIATCompileErrorC *error);
void hiat_compiler_free(HIATCompiler *compiler);

size_t hiat_script_data_get_warnings(const HIATCompiledScriptData *script_data, HIATCompileErrorC *warnings);
void hiat_script_data_free(HIATCompiledScriptData *compiler);

void hiat_error_free(HIATCompileErrorC *error);

typedef enum HIATValueType {
    HIAT_Unparsed,
    HIAT_SpecialForm,
    HIAT_FunctionName,
    HIAT_Passthrough,
    HIAT_Void,
    HIAT_Boolean,
    HIAT_Real,
    HIAT_Short,
    HIAT_Long,
    HIAT_String,
    HIAT_Script,
    HIAT_TriggerVolume,
    HIAT_CutsceneFlag,
    HIAT_CutsceneCameraPoint,
    HIAT_CutsceneTitle,
    HIAT_CutsceneRecording,
    HIAT_DeviceGroup,
    HIAT_Ai,
    HIAT_AiCommandList,
    HIAT_StartingProfile,
    HIAT_Conversation,
    HIAT_Navpoint,
    HIAT_HudMessage,
    HIAT_ObjectList,
    HIAT_Sound,
    HIAT_Effect,
    HIAT_Damage,
    HIAT_LoopingSound,
    HIAT_AnimationGraph,
    HIAT_ActorVariant,
    HIAT_DamageEffect,
    HIAT_ObjectDefinition,
    HIAT_GameDifficulty,
    HIAT_Team,
    HIAT_AiDefaultState,
    HIAT_ActorType,
    HIAT_HudCorner,
    HIAT_Object,
    HIAT_Unit,
    HIAT_Vehicle,
    HIAT_Weapon,
    HIAT_Device,
    HIAT_Scenery,
    HIAT_ObjectName,
    HIAT_UnitName,
    HIAT_VehicleName,
    HIAT_WeaponName,
    HIAT_DeviceName,
    HIAT_SceneryName
} HIATValueType;

typedef enum HIATScriptType {
    HIAT_Startup,
    HIAT_Dormant,
    HIAT_Continuous,
    HIAT_Static,
    HIAT_Stub
} HIATScriptType;

typedef enum HIATNodeTypeC {
    HIAT_Primitive,
    HIAT_Global,
    HIAT_FunctionCall,
    HIAT_ScriptCall
} HIATNodeTypeC;

typedef union HIATScriptNodeDataC {
    size_t offset;
    float real;
    int32_t long_int;
    int16_t short_int;
    bool boolean;
} HIATScriptNodeDataC;

typedef struct HIATScriptNodeC {
    const char *file;
    size_t line;
    size_t column;
    const char *string_data;
    uint16_t index_union;
    HIATValueType value_type;
    HIATNodeTypeC node_type;
    HIATScriptNodeDataC node_data;
    size_t next_node;
} HIATScriptNodeC;

size_t hiat_script_data_get_nodes(const HIATCompiledScriptData *script_data, HIATScriptNodeC *nodes);

typedef struct HIATScriptC {
    const char *name;
    const char *file;
    size_t line;
    size_t column;
    HIATScriptType script_type;
    HIATValueType return_type;
    size_t first_node;
} HIATScriptC;

typedef struct HIATGlobalC {
    const char *name;
    const char *file;
    size_t line;
    size_t column;
    HIATValueType value_type;
    size_t first_node;
} HIATGlobalC;

size_t hiat_script_data_get_scripts(const HIATCompiledScriptData *script_data, HIATScriptC *scripts);
size_t hiat_script_data_get_globals(const HIATCompiledScriptData *script_data, HIATGlobalC *globals);

#ifdef __cplusplus
}
#endif

#endif
