# Athena — Autonomous LLM Experimentation Laboratory

> **"Athena no ejecuta configuraciones; Athena descubre configuraciones."**

Athena es un laboratorio autónomo para evaluar, comparar y optimizar LLMs locales. No es un benchmark, no es un launcher, no es un wrapper de llama.cpp. Es un **sistema que diseña experimentos, ejecuta inferencia real, mide telemetría completa y produce conocimiento reproducible** sobre cualquier modelo GGUF en cualquier hardware con GPU NVIDIA.

---

## Filosofía

Athena se basa en cuatro principios:

1. **Nada se estima — todo se mide.** No hay heurísticas ni `estimate()`. Cada decisión se basa en evidencia obtenida mediante ejecución real.
2. **Cada experimento responde una pregunta científica.** No hay "benchmarks" genéricos. Cada prueba tiene una hipótesis y produce una conclusión verificable.
3. **El conocimiento es acumulativo.** Los resultados se persisten como evidencia. Las configuraciones que fallan se registran para no repetirse.
4. **Los agentes consumen artefactos, no texto.** Cualquier agente (Buffy, Hermes, Claude Code, Codex) puede leer fases individuales sin ejecutar nada.

---

## Arquitectura

```
Study (Programa Científico)
  │
  ├── Campaign (Conjunto de experimentos)
  │     │
  │     ├── Experiment (Configuración única)
  │     │     │
  │     │     ├── Phase 1 → Hardware Discovery
  │     │     ├── Phase 2 → Runtime Discovery
  │     │     ├── Phase 3 → Runtime Capabilities
  │     │     ├── Phase 4 → GGUF Inspection
  │     │     ├── Phase 5 → Memory Hypothesis
  │     │     ├── Phase 6 → Execution Laboratory 🔬 (inferencia real)
  │     │     ├── Phase 7 → Runtime Fingerprint
  │     │     ├── Phase 8 → Capability Discovery
  │     │     ├── Phase 9 → Parameter Normalization
  │     │     ├── Phase 10 → Output Validation
  │     │     └── Phase 11 → Experiment Validation 🛡️
  │     │
  │     └── Evidence (Resultados y artefactos)
  │
  └── Knowledge Base (Conocimiento acumulado)
```

### Phase Pipeline — 11 fases científicas

Cada fase responde **UNA** pregunta. No más.

| Fase | Pregunta | Requiere |
|------|----------|----------|
| **PHASE-0001** Hardware | ¿Qué hardware existe? | — |
| **PHASE-0002** Runtime Discovery | ¿Qué runtimes existen? | 0001 |
| **PHASE-0003** Runtime Capabilities | ¿Qué capacidades reales tiene cada runtime? | 0002 |
| **PHASE-0004** GGUF Inspection | ¿Qué dice realmente este modelo? | `--model` |
| **PHASE-0005** Memory Hypothesis | ¿Qué configuraciones parecen posibles? | `--model` |
| **PHASE-0006** Execution Laboratory | ¿Funciona esta configuración realmente? | `--model` + `--runtime` |
| **PHASE-0007** Runtime Fingerprint | ¿Qué es realmente este runtime? | `--runtime` |
| **PHASE-0008** Capability Discovery | ¿Qué capacidades declara realmente este runtime? | `--runtime` |
| **PHASE-0009** Parameter Normalization | ¿Cuál es el conjunto común de parámetros? | — |
| **PHASE-0010** Output Validation | ¿La salida obtenida es válida? | 0006 |
| **PHASE-0011** Experiment Validation | ¿Este experimento merece ejecutarse? 🛡️ | — |

---

## Scientific Programs (Estudios)

Los estudios son programas científicos completos. Combina múltiples fases para responder una pregunta compleja.

### SP-005: Runtime Health Check

```bash
ath study SP-005
```

Verifica que todos los runtimes detectados carguen correctamente, generen tokens y finalicen sin errores. Es la puerta de entrada del laboratorio.

### PC-001: Runtime Comparison

```bash
ath study PC-001
```

Compara todos los runtimes detectados (Official, TurboQuant, PrismML) bajo exactamente los mismos parámetros. Cada runtime ejecuta 5 repeticiones. Mide load time, first token, TPS, VRAM, RAM y estabilidad.

---

## Instalación

### Requisitos

- **Rust** 2024 edition (`curl https://sh.rustup.rs -sSf | sh`)
- **CUDA** 12.x+ y NVIDIA driver (`nvidia-smi` debe funcionar)
- **llama.cpp** (al menos un build con `llama-server`)
- **Modelo GGUF** (por ejemplo, Qwen 3.5 4B Q4_K_M)

### Compilar

```bash
git clone <repo>
cd athenas

# Release build (recomendado para inferencia)
cargo build --release --manifest-path crates/athenas-compiler/Cargo.toml

# El binario se encuentra en:
# crates/athenas-compiler/target/release/ath
```

### Verificar instalación

```bash
./crates/athenas-compiler/target/release/ath doctor
```

Debe detectar GPU, VRAM, RAM, CPU, runtimes (llama.cpp, TurboQuant, PrismML, Ollama, etc.) y modelos GGUF.

---

## Uso básico

### Descubrir hardware y runtimes

```bash
# Detectar todo
ath doctor

# Listar las 11 fases disponibles
ath phase list
```

### Ejecutar una fase individual

```bash
# Hardware
ath phase run PHASE-0001-hardware

# Runtime Discovery
ath phase run PHASE-0002-runtime-discovery

# Fingerprint de un runtime específico
ath phase run PHASE-0007-runtime-fingerprint \
  --runtime /path/to/llama-server

# Inspeccionar un modelo GGUF
ath phase run PHASE-0004-gguf-inspection \
  --model /path/to/model.gguf

# Ejecutar inferencia real (PHASE-0006)
ath phase run PHASE-0006-execution-lab \
  --runtime /path/to/llama-server \
  --model /path/to/model.gguf
```

### Ejecutar un estudio científico completo

```bash
# Health Check
ath study SP-005

# Runtime Comparison (tarda más)
ath study PC-001
```

### Gestionar experimentos

```bash
# Añadir experimento a la cola
ath queue add --model /path/to/model.gguf

# Procesar siguiente experimento
ath queue process

# Ver estado de la cola
ath queue list

# Ver detalle de un experimento
ath queue show --experiment EXP-1234567890

# Limpiar experimentos antiguos
ath queue clean --days 7
```

### Analizar un modelo

```bash
ath analyze /path/to/model.gguf
```

### Recomendar configuración

```bash
ath recommend "Rust Development"
ath recommend "Frontend"
ath recommend "Web Pentest"
```

---

## Cómo añadir un runtime nuevo

Athena detecta runtimes automáticamente buscando en `$PATH` y directorios de compilación comunes (`~/llama.cpp/build/bin/`, `~/prism-llama.cpp/build/bin/`, etc.).

Para añadir un runtime manualmente, basta con que el binario `llama-server` esté accesible. Athena lo descubrirá en la próxima ejecución de `PHASE-0002-runtime-discovery` o `ath doctor`.

Si el runtime tiene capacidades especiales (TurboQuant, Bonsai, ISWA, etc.), Athena las detecta automáticamente analizando la salida de `--help`.

---

## Cómo añadir un modelo nuevo

Athena busca modelos GGUF en directorios comunes (`~/models/`, `~/AI/`, `~/Downloads/`).

Para añadir un modelo manualmente:

```bash
# Coloca el GGUF en cualquier directorio
mkdir -p ~/models/qwen3.5
# Copia o descarga el modelo allí

# Athena lo detectará con:
ath doctor
# o
ath phase run PHASE-0002-runtime-discovery
```

Para inspeccionar un modelo específico:

```bash
ath phase run PHASE-0004-gguf-inspection --model ~/models/tu-modelo.gguf
```

---

## Cómo crear un Programa Científico

Los programas científicos se definen en `crates/athenas-compiler/src/runtime/study/mod.rs`, en la función `built_in_studies()`.

Cada estudio necesita:

```rust
Study {
    id: "PC-002",                          // ID único
    question: "¿Pregunta científica?",      // Pregunta que responde
    phase_ids: vec!["PHASE-0001", ...],     // Fases necesarias
    repetitions: 5,                         // Repeticiones por runtime
    default_context: 32768,                 // Contexto por defecto
    default_max_tokens: 100,               // Tokens por defecto
    objective: "maximum_quality",           // Objetivo
    success_criteria: vec!["no_oom", ...], // Criterios de éxito
    validate_before_execution: true,       // Validar antes de ejecutar
    store_evidence: true,                  // Almacenar evidencia
}
```

---

## Archivos que produce Athena

```
.state/
  experiments/
    EXP-<timestamp>/
      phases/
        PHASE-0001-hardware/
          artifact.json      ← Datos estructurados
          metrics.json       ← Métricas numéricas
          timeline.json       ← Línea de tiempo de eventos
          evidence/           ← Evidencia adicional
      report.json            ← Reporte completo del experimento

  queue/
    queue.json               ← Estado de la cola de experimentos

  evidence/
    negative/                ← Experimentos rechazados (nunca repetir)
      NEG-<timestamp>/
        evidence.json
    positive/                ← Experimentos exitosos
      POS-<timestamp>/
        evidence.json
    index.json               ← Índice de búsqueda rápida
```

Cada `artifact.json` contiene:

- **phase_id**: Identificador único de la fase
- **status**: Success / Failure / Skipped
- **metrics**: Valores numéricos con unidades
- **timeline**: Eventos durante la ejecución
- **duration_ms**: Duración real
- **raw_log_path**: Ruta a logs sin procesar

---

## Visión

Athena no busca el mejor benchmark.

Athena busca **descubrir conocimiento reproducible** sobre cómo se comportan los modelos locales en hardware real.

Cada experimento es una oportunidad de aprender algo que no sabíamos. Cada resultado fallido es evidencia valiosa que evita repetir errores. Cada nuevo runtime o modelo es un organismo vivo que Athena debe estudiar, no un simple binario que ejecutar.

> Athena no es un producto terminado. Es un laboratorio que aprende.

---

## Licencia

MIT
