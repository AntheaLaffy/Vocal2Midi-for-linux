# Acknowledgements

Vocal2Midi builds on several open-source projects for transcription, alignment, pitch extraction, runtime support, and UI tooling.

## Repository license

The overall Vocal2Midi repository is distributed under the **Apache License 2.0**.

Third-party components, vendored code, bundled data, dictionaries, and other embedded materials may retain their own original licenses, copyright notices, and attribution requirements. Where such notices are present, they remain applicable to the corresponding materials.

## Core upstream projects

| Project | Role in Vocal2Midi | Upstream repository |
| --- | --- | --- |
| GAME | note and pitch extraction | [openvpi/GAME](https://github.com/openvpi/GAME) |
| HubertFA | phoneme-level forced alignment | [wolfgitpr/HubertFA](https://github.com/wolfgitpr/HubertFA) |
| LyricFA | lyric matching and G2P-based lyric alignment helpers | [wolfgitpr/LyricFA](https://github.com/wolfgitpr/LyricFA) |
| FunASR | broader ASR foundation referenced by the Qwen3-ASR integration path | [modelscope/FunASR](https://github.com/modelscope/FunASR) |
| llama.cpp | CPU decoder runtime used by the Qwen3-ASR DML path | [ggml-org/llama.cpp](https://github.com/ggml-org/llama.cpp) |
| ONNX Runtime | DirectML and CPU inference execution | [microsoft/onnxruntime](https://github.com/microsoft/onnxruntime) |
| PyQt-Fluent-Widgets | Fluent-style desktop UI components | [zhiyiYo/PyQt-Fluent-Widgets](https://github.com/zhiyiYo/PyQt-Fluent-Widgets) |
| CPython / Python Software Foundation | reference algorithm for the Rust adaptation of Python 3.12.13 list-sort comparison scheduling and Unicode 15.0 printability | [python/cpython v3.12.13](https://github.com/python/cpython/tree/v3.12.13) |

## Vendored components in this repository

The repository currently includes local copies or adapted subsets of some upstream projects:

- `inference/HubertFA/`
- `inference/LyricFA/`
- `inference/qwen3asr_dml/gguf/`

These copies may contain project-specific edits for integration, runtime changes, or interface compatibility.

## Additional libraries

Vocal2Midi also relies on widely used libraries from the Python audio and scientific stack, including `librosa`, `numpy`, `scipy`, `soundfile`, `textgrid`, `PyYAML`, and related tooling listed in `requirements.txt`.

## Thanks

Thanks to the maintainers and contributors of the upstream projects above, and to the singing synthesis / music information retrieval community whose open tooling makes this kind of integration work possible.
