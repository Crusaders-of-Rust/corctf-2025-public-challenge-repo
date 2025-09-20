// pid.c - C reimplementation of Python PID Controller and high-level Controller for WASM
// Build: see Makefile (clang -> wasm32, no stdlib, single exported function)
// Exported API:
//   uint64_t controller_step(double sp, double vC1, double vC2, double iL)
// Returns u0 and u1 packed as two IEEE-754 float32 values into a single uint64:
//   return ((uint64_t)u0_bits << 32) | u1_bits;
// This avoids exporting linear memory and keeps the module with a single export.

#include <stdint.h>

// Visibility: hide everything by default; explicitly export controller_step via linker flag
#ifdef __GNUC__
#pragma GCC visibility push(hidden)
#endif

// PID structure and step implementation matching Python logic
typedef struct {
    double kf, kp, ki, kd;
    double err_prev;
    double integral;
} PID;

static inline double pid_step(PID *pid, double target, double actual) {
    double err = target - actual;
    double result = pid->kf * target + pid->kp * err + pid->integral + pid->kd * (err - pid->err_prev);
    pid->err_prev = err;
    pid->integral += err * pid->ki;
    return result;
}

// distribute() as in ctrl.py
static inline void singledir_distribute(double u, double out[2]) {
    if (u >= 1.0) {
        out[0] = 1.0;
        out[1] = 1.0 - (u - 1.0);
    } else {
        out[0] = u;
        out[1] = 1.0;
    }
}

static inline void distribute(double u, double out[2]) {
    if (u >= 0.0) {
        singledir_distribute(u, out);
    } else {
        double tmp[2];
        singledir_distribute(-u, tmp);
        out[0] = tmp[1];
        out[1] = tmp[0];
    }
}

// Persistent controllers (match Python persistent state)
static PID voltage_controller = { .kf = 0.0, .kp = 0.2, .ki = 0.15, .kd = 0.0, .err_prev = 0.0, .integral = 0.0 };
static PID current_controller = { .kf = 0.1, .kp = 0.01, .ki = 0.01, .kd = 0.0, .err_prev = 0.0, .integral = 0.0 };

// Pack two float32 values into one uint64 (u0 high 32 bits, u1 low 32 bits)
static inline uint64_t pack_f32_pair(double u0, double u1) {
    union { float f; uint32_t u; } a, b;
    a.f = (float)u0;
    b.f = (float)u1;
    return ((uint64_t)a.u << 32) | (uint64_t)b.u;
}

#ifdef __GNUC__
#pragma GCC visibility pop
#endif

// Exported function: compute controller output given sp and plant state (vC1, vC2, iL)
// vC1 is unused in current Python logic but kept for signature compatibility/future use.
// We deliberately avoid any memory arguments to keep the module with one export.
__attribute__((visibility("default")))
unsigned long long controller_step(double target_voltage, double vC1, double vC2, double iL) {
    (void)vC1; // unused currently
    double target_current = pid_step(&voltage_controller, target_voltage, vC2);
    double ctrl_current = pid_step(&current_controller, target_current, iL);

    double u[2];
    distribute(ctrl_current, u);

    return (unsigned long long)pack_f32_pair(u[0], u[1]);
}
