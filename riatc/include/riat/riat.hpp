#ifndef RAT_IN_A_TUBE_HPP
#define RAT_IN_A_TUBE_HPP

#include "riat.h"

#include <cstdio>
#include <string>
#include <exception>
#include <memory>
#include <vector>
#include <optional>

namespace RIAT {
    class CompileError : public std::exception {
    public:
        CompileError(const RIATCompileErrorC &error, const char *type) noexcept {
            this->reason = error.message;
            this->file = error.file;
            this->line = error.line;
            this->column = error.column;

            char what_error_c[512];
            std::snprintf(what_error_c, sizeof(what_error_c), "%s:%zu:%zu: %s: %s", this->file.c_str(), this->line, this->column, type, this->reason.c_str());
            what_error = what_error_c;
        };
        const char *what() const noexcept override {
            return this->what_error.c_str();
        }
        std::size_t get_line() const noexcept {
            return this->line;
        }
        std::size_t get_column() const noexcept {
            return this->column;
        }
        const char *get_file() const noexcept {
            return this->file.c_str();
        }
        const char *get_reason() const noexcept {
            return this->reason.c_str();
        }
        ~CompileError() noexcept override {}
    private:
        std::string what_error;
        std::size_t line;
        std::size_t column;
        std::string file;
        std::string reason;
    };

    class CompilerScriptResult {
    public:
        /**
         * Get all scripts.
         *
         * @return scripts
         */
        std::vector<RIATScriptC> get_scripts() {
            if(!this->script_data) {
                return {};
            }

            auto *script_data = this->script_data.get();
            auto script_count = ::riat_script_data_get_scripts(script_data, nullptr);
            std::vector<RIATScriptC> r(script_count);
            ::riat_script_data_get_scripts(script_data, r.data());
            return r;
        }

        /**
         * Get all parameters for the given script.
         *
         * @return parameters
         */
        std::vector<RIATScriptParameterC> get_script_parameters_for_script(const RIATScriptC &script) {
            if(!this->script_data) {
                return {};
            }

            auto *script_data = this->script_data.get();
            auto parameter_count = ::riat_script_data_get_script_parameters(script_data, script.name, nullptr);
            std::vector<RIATScriptParameterC> r(parameter_count);
            ::riat_script_data_get_script_parameters(script_data, script.name, r.data());
            return r;
        }

        /**
         * Get all globals.
         *
         * @return globals
         */
        std::vector<RIATGlobalC> get_globals() {
            if(!this->script_data) {
                return {};
            }

            auto *script_data = this->script_data.get();
            auto global_count = ::riat_script_data_get_globals(script_data, nullptr);
            std::vector<RIATGlobalC> r(global_count);
            ::riat_script_data_get_globals(script_data, r.data());
            return r;
        }

        /**
         * Get all nodes.
         *
         * @return nodes
         */
        std::vector<RIATScriptNodeC> get_nodes() {
            if(!this->script_data) {
                return {};
            }

            auto *script_data = this->script_data.get();
            auto node_count = ::riat_script_data_get_nodes(script_data, nullptr);
            std::vector<RIATScriptNodeC> r(node_count);
            ::riat_script_data_get_nodes(script_data, r.data());
            return r;
        }

        /**
         * Get all warnings.
         *
         * @return warnings
         */
        std::vector<CompileError> get_warnings() {
            if(!this->script_data) {
                return {};
            }

            std::vector<CompileError> r;
            auto *script_data = this->script_data.get();
            auto warning_count = ::riat_script_data_get_warnings(script_data, nullptr);
            r.reserve(warning_count);

            std::vector<RIATCompileErrorC> errors;
            errors.resize(warning_count);
            ::riat_script_data_get_warnings(script_data, errors.data());

            for(auto &e : errors) {
                r.emplace_back(e, "warning");
            }
            return r;
        }

        CompilerScriptResult(const CompilerScriptResult &) = delete;
        CompilerScriptResult(CompilerScriptResult &&) = default;
        CompilerScriptResult &operator=(CompilerScriptResult &&) = default;

        /** Instantiate an instance using a smart pointer */
        CompilerScriptResult(std::unique_ptr<RIATCompiledScriptData, void(*)(RIATCompiledScriptData*)> &&script_data) : script_data(std::move(script_data)) {}

        /** Instantiate an empty instance */
        CompilerScriptResult() : CompilerScriptResult(std::unique_ptr<RIATCompiledScriptData, void(*)(RIATCompiledScriptData*)>(nullptr, ::riat_script_data_free)) {};

    private:
        std::unique_ptr<RIATCompiledScriptData, void(*)(RIATCompiledScriptData*)> script_data;
    };

    class Compiler {
    public:
        /**
         * Load the given script
         *
         * @param script_source_data   pointer to the script source data
         * @param script_source_length length of the script source data
         * @param file_name            name of the file (for error reporting)
         *
         * @throws RIAT::CompileError on failure
         */
        void read_script_data(const std::uint8_t *script_source_data, std::size_t script_source_length, const char *file_name) {
            RIATCompileErrorC error;
            if(::riat_compiler_read_script_data(this->get_instance(), file_name, script_source_data, script_source_length, &error) != 0) {
                auto exception = CompileError(error, "error");
                ::riat_error_free(&error);
                throw exception;
            }
        }

        /**
         * Compile the given script and, if successful, clear all loaded scripts.
         *
         * @throws RIAT::CompileError on failure
         */
        CompilerScriptResult compile_scripts() {
            RIATCompileErrorC error;
            auto new_compiled_data = std::unique_ptr<RIATCompiledScriptData, void(*)(RIATCompiledScriptData*)>(::riat_compiler_compile_script_data(this->get_instance(), &error), ::riat_script_data_free);
            if(new_compiled_data.get() == nullptr) {
                auto exception = CompileError(error, "error");
                ::riat_error_free(&error);
                throw exception;
            }
            return CompilerScriptResult(std::move(new_compiled_data));
        }

        /**
         * Get the instance handle
         *
         * @return instance
         */
        RIATCompiler *get_instance() noexcept {
            return this->instance.get();
        }

        /**
         * Instantiate a compiler instance
         *
         * @param target   target engine
         */
        Compiler(RIATCompileTarget target) : instance(::riat_compiler_new(target), ::riat_compiler_free) {
            if(this->instance.get() == nullptr) {
                throw std::exception();
            }
        }
    private:
        std::unique_ptr<RIATCompiler, void(*)(RIATCompiler*)> instance;
    };
}

#endif
