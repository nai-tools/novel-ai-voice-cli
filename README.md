# NovelAI Voice CLI
A CLI application using the novelai-api package to export TTS audio files

# Usage
```bash
Usage: novel-ai-voice-cli [OPTIONS] --token <TOKEN> --file <FILE>

Options:
  -t, --token <TOKEN>                        
  -f, --file <FILE>                          Input File path
  -v, --voice <VOICE>                        TTS Voice Seed [default: Aini]
  -p, --parallel-threads <PARALLEL_THREADS>  Number of "Threads" to use for downloading [default: 10]
  -h, --help                                 Print help
  -V, --version                              Print version
```

# Installation
```bash
git clone https://github.com/nai-tools/novel-ai-voice-cli
cd novel-ai-voice-cli
cargo install --path .

# Use application
novel-ai-voice-cli [OPTIONS] --token <TOKEN> --file <FILE>
```