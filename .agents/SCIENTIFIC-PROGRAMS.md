# Athena Scientific Programs v2

## Principio fundamental

Athena no ejecuta benchmarks. Athena ejecuta **Programas Científicos**.

Cada programa responde una **pregunta permanente**. La respuesta evoluciona con la evidencia acumulada. Nunca se "termina" — se actualiza.

---

## Reglas

1. **1 pregunta por programa** — Si responde dos, está mal diseñado.
2. **Repeticiones variables** — No un número fijo. Ejecuta hasta alcanzar **Evidence Saturation** (la varianza no cambia significativamente). Máximo configurable.
3. **Evidence Saturation** — Cuando nuevas mediciones no cambian la conclusión, detenerse. Criterio: coeficiente de variación < 5% en las últimas 3 ejecuciones o máximo 10 repeticiones.
4. **Variables controladas** — Todo lo que no es la variable independiente debe permanecer igual.
5. **Normalización** — Antes del experimento, verificar que todos los runtimes usan exactamente los mismos parámetros compatibles.
6. **Calibración** — Antes de la campaña, un experimento corto con cada runtime para verificar estabilidad.
7. **Reproducible** — Cada experimento incluye huella completa del entorno.
8. **Confianza** — Cada conclusión reporta confianza basada en varianza y cantidad de evidencia.
9. **Respuestas versionadas** — Cada respuesta tiene revision, fecha y condiciones de validez.
10. **Objetivo declarado** — Cada programa declara su función objetivo (maximizar/minimizar/restricciones).

---

## Estructura de un Programa Científico

```yaml
ScientificProgram:
  id: PC-001
  title: "Runtime Comparison"
  question: "¿Cuál runtime ofrece el mejor rendimiento para un modelo dado en un hardware dado?"
  
  permanent: true  # La pregunta nunca desaparece; la respuesta evoluciona
  
  objective:
    maximize: [throughput, stability]
    minimize: [latency, vram_peak]
    constraints:
      - min_tokens_per_second: 12
      - no_oom: true
      - min_confidence: 0.95
  
  independent_variable: runtime_variant
  
  controlled_variables:
    model: qwen3.5-4b-q4_k_m.gguf
    context: 32768
    gpu_layers: 999
    batch: 512
    ubatch: 256
    threads: 8
    kv_cache: f16
    flash_attention: true
    prompt: "Hello, explain what you are."
    seed: 42
    temperature: 0.7
  
  sub_questions:
    - PC-001A: "¿Todos cargan correctamente?"
    - PC-001B: "¿Cuánto tardan en cargar?"
    - PC-001C: "¿Cuál produce el primer token antes?"
    - PC-001D: "¿Cuál mantiene mayor TPS?"
    - PC-001E: "¿Cuál consume menos VRAM?"
    - PC-001F: "¿Cuál consume menos RAM?"
    - PC-001G: "¿Cuál presenta mayor estabilidad?"
    - PC-001H: "¿Existe alguna incompatibilidad?"
  
  repetitions:
    strategy: adaptive  # Hasta evidence saturation
    min: 3
    max: 10
    saturation_criteria: coefficient_of_variation < 0.05  # Últimas 3 ejecuciones
  
  calibration:
    enabled: true  # Experimento corto antes de la campaña
    max_tokens: 30
    purpose: "Verificar que el runtime carga, genera y no falla"
```

---

## Ciclo de vida de una respuesta

```yaml
AnswerRevision:
  program: PC-001
  revision: 14
  generated_at: 2026-07-17T18:30:00Z
  confidence: 0.946
  evidence_count: 78
  
  valid_for:
    gpu: "RTX3050 Laptop 6GB"
    driver: "595.84"
    cuda: "13.2"
    kernel: "6.19.14-kali-amd64"
    runtime_set: [Official, TurboQuant, PrismML]
    model: "qwen3.5-4b-q4_k_m.gguf"
  
  conclusion: |
    TurboQuant supera a Official en tokens/s en +18.4%
    PrismML ofrece la latencia más baja del primer token
    
  status: current  # current, outdated (environment changed), superseded (new revision)
  
  derived_outputs:
    rankings:  # JSON, para agentes
    tables:
    curves:
    histograms:
```

Si cualquier condición de `valid_for` cambia (driver nuevo, runtime actualizado), la respuesta pasa automáticamente a `outdated`. No incorrecta — desactualizada.

---

## Evidence Saturation

Criterio para dejar de ejecutar repeticiones:

```
saturation: coefficient_of_variation(tokens_per_second, last_3) < 0.05
```

Es decir, cuando las últimas 3 mediciones tienen un coeficiente de variación menor a 5%, detenerse. Mínimo 3 repeticiones, máximo 10.

Si la varianza es alta (ej: 22, 31, 28, 35, 24 TPS), Athena sigue ejecutando hasta alcanzar saturación o el máximo.

---

## Objective Function

Cada programa declara:

```yaml
objective:
  maximize: [throughput, stability]
  minimize: [latency, vram_peak]
  constraints:
    min_tokens_per_second: 12
    no_oom: true
```

El índice compuesto se calcula como:

```
Score = w1 * throughput_norm + w2 * stability_norm - w3 * latency_norm - w4 * vram_norm
```

Donde `_norm` indica normalización (0-1) y `wi` son pesos configurables.

---

## Sub-preguntas (PC-001)

PC-001 se compone de 8 sub-preguntas independientes:

| ID | Pregunta | Métrica |
|---|---|---|
| PC-001A | ¿Todos cargan correctamente? | éxito/fallo en load |
| PC-001B | ¿Cuánto tardan en cargar? | load_time_s |
| PC-001C | ¿Cuál produce el primer token antes? | first_token_ms |
| PC-001D | ¿Cuál mantiene mayor TPS? | tokens_per_second |
| PC-001E | ¿Cuál consume menos VRAM? | vram_peak_gb |
| PC-001F | ¿Cuál consume menos RAM? | ram_peak_gb |
| PC-001G | ¿Cuál presenta mayor estabilidad? | variance de TPS |
| PC-001H | ¿Existe alguna incompatibilidad? | errores, crashes, OOM |

PC-001 no es una conclusión única. Es la composición de 8 respuestas independientes.

---

## Calibración

Antes de PC-001, ejecutar un experimento de **calibración** con cada runtime:

```yaml
Calibration:
  max_tokens: 30
  repetitions: 1
  purpose: "Verificar que el runtime carga, genera y no falla"
  
  checks:
    - load_success
    - generation_completes
    - no_oom
    - no_crash
    - expected_output: "contiene texto"
```

Si algún runtime falla la calibración, se detiene la campaña y se reporta el error. No se pierden horas de GPU.

---

## Normalización

Antes de comparar, verificar que todos los runtimes usan exactamente:

- mismo modelo (mismo hash)
- mismo prompt (mismo hash)
- mismo seed
- misma temperatura
- mismo contexto
- mismo batch y ubatch
- mismas GPU layers
- mismos parámetros de KV cache
- mismo flash attention

Si un runtime no soporta un parámetro, se registra explícitamente como **diferencia experimental conocida**. No se oculta.

---

## Identidad del Runtime (no ruta)

Los resultados se almacenan por identidad, no por ruta:

```yaml
RuntimeIdentity:
  family: llama.cpp
  variant: official       # official, turboquant, prismml
  commit: "505b1ed"
  build_flags: "-DGGML_CUDA=ON"
```

La ruta del binario solo sirve para ejecutar. El conocimiento se construye sobre la identidad.

---

## Derivados automáticos

Cada programa científico produce automáticamente:

- **Ranking** por cada métrica (TPS, VRAM, latencia)
- **Tabla comparativa** (runtime × métrica)
- **Curvas** de contexto vs rendimiento
- **Histogramas** de dispersión por runtime
- **Comparación de confianza** entre runtimes

Todo en JSON. Para agentes, no para humanos.

---

## Programas Científicos v2

### PC-001: Runtime Comparison

| Campo | Valor |
|---|---|
| **Pregunta** | ¿Cuál runtime ofrece el mejor comportamiento para un mismo modelo en un mismo hardware? |
| **Variable independiente** | Runtime (Official, TurboQuant, PrismML) |
| **Sub-preguntas** | PC-001A a PC-001H |
| **Repeticiones** | Adaptativas (3-10, hasta saturación) |
| **Calibración** | 30 tokens por runtime antes de la campaña |
| **Total experimentos estimado** | 9-30 (3 runtimes × 3-10 reps) |
| **Objetivo** | Maximizar: throughput, estabilidad. Minimizar: latencia, VRAM |

### PC-002: Context Frontier

Búsqueda adaptativa del contexto máximo estable.

### PC-003: GPU Layer Frontier

ngl vs rendimiento.

### PC-004: Memory Strategy

All VRAM vs Buendia Turbo3 vs Hybrid vs CPU vs KV Hybrid.

### PC-005: KV Cache Quantization

f16 vs q8 vs q6 vs q5 vs q4 vs turbo3.

### PC-006: Batch Search

batch 64-1024.

### PC-007: Thread Scaling

threads 2-8.

### PC-008: Prompt Scaling

prompt 64-16384 tokens.

### PC-009: Runtime Features

Flash Attention ON/OFF, Speculative ON/OFF.

### PC-010: Model Scaling

4B vs 9B vs 27B.
