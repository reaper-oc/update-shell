#!/usr/bin/env python3
"""Generate registry packages for updSH."""

import json
import os
import subprocess

ASCII_FILE = os.path.join(os.path.dirname(__file__), "ascii.txt")
OUTPUT_RUST = os.path.join(os.path.dirname(__file__), "src", "builtin_packages.json")
OUTPUT_VERCEL = os.path.join(os.path.dirname(__file__), "registry", "api", "packages.json")

packages = []

# ── Helpers ──────────────────────────────────────────────────────

def source(name, desc, content, author="updSH", version="1.0.0", deps=None, env=None):
    packages.append({
        "name": name, "version": version, "description": desc,
        "author": author, "type": "source", "depends": deps or [],
        "env": env or {},
        "files": [{"path": f"{name}.sh", "content": content}]
    })

def compile_c(name, desc, code, binary=None, author="updSH", version="1.0.0"):
    packages.append({
        "name": name, "version": version, "description": desc,
        "author": author, "type": "compile", "language": "c",
        "binary": binary or name, "depends": [],
        "env": {}, "source": code,
        "files": []
    })

def compile_rust(name, desc, code, binary=None, author="updSH", version="1.0.0"):
    packages.append({
        "name": name, "version": version, "description": desc,
        "author": author, "type": "compile", "language": "rust",
        "binary": binary or name, "depends": [],
        "env": {}, "source": code,
        "files": []
    })

def build_pkg(name, desc, url, binary, build_type="cmake",
              author="updSH", version="2.0.0", files=None, deps=None, env=None):
    packages.append({
        "name": name, "version": version, "description": desc,
        "author": author, "type": "build",
        "binary": binary, "depends": deps or [],
        "env": env or {},
        "source_url": url, "build_type": build_type,
        "files": files or []
    })

# ═══════════════════════════════════════════════════════════════════
#  SOURCE PACKAGES — alias collections
# ═══════════════════════════════════════════════════════════════════

source("git-aliases", "Useful Git aliases: gst, gd, gc, gb, glog, etc.", """\
alias gst='git status'
alias gd='git diff'
alias gdc='git diff --cached'
alias gc='git commit'
alias gcm='git commit -m'
alias gca='git commit --amend'
alias gb='git branch'
alias gco='git checkout'
alias gl='git log --oneline --graph'
alias glog='git log --oneline --graph --all'
alias gp='git push'
alias gpl='git pull'
alias ga='git add'
alias gap='git add -p'
alias gr='git remote -v'
alias gstash='git stash'
alias gstashp='git stash pop'
""")

source("docker-aliases", "Docker shortcuts: dps, dlog, dexec, dprune", """\
alias dps='docker ps'
alias dpa='docker ps -a'
alias dlog='docker logs'
alias dlogs='docker logs -f'
alias dexec='docker exec -it'
alias dimg='docker images'
alias dprune='docker system prune -af'
alias dstop='docker stop $(docker ps -q)'
alias drm='docker rm $(docker ps -aq)'
alias dcomp='docker compose'
""")

source("ls-improved", "Better ls with color and human-readable sizes", """\
alias ls='ls --color=auto'
alias l='ls -CF'
alias la='ls -A'
alias ll='ls -lhF'
alias lla='ls -lAhF'
alias lt='ls -ltrhF'
alias ldot='ls -ld .*'
""")

source("safe-rm", "Interactive aliases for rm, cp, mv to prevent accidents", """\
alias rm='rm -i'
alias cp='cp -i'
alias mv='mv -i'
alias rmr='rm -rf'
alias trash='mv --backup=numbered -t ~/.trash'
""")

source("navigation", "Directory navigation helpers: up, back, md, dot-dot shortcuts", """\
up() { local d=''; limit="${1:-1}"; for ((i=0;i<limit;i++)); do d="$d/.."; done; d=$(echo "$d" | sed 's|^/||'); [ -n "$d" ] && cd "$d" || true; }
back() { cd "$OLDPWD" || true; }
..() { cd ..; }
...() { cd ../..; }
....() { cd ../../..; }
.....() { cd ../../../..; }
md() { mkdir -p "$1" && cd "$1"; }
""")

source("sysinfo", "Quick system info aliases: myip, cpu, mem, disk, ports", """\
alias myip='curl -s ifconfig.me 2>/dev/null || echo no internet'
alias cpu='lscpu | head -20'
alias mem='free -h'
alias disk='df -h'
alias ports='ss -tulanp'
alias psmy='ps aux | grep $USER'
alias fs='df -h .'
""")

source("colorize", "Color and grep aliases with syntax highlighting", """\
alias grep='grep --color=auto'
alias egrep='egrep --color=auto'
alias fgrep='fgrep --color=auto'
alias diff='diff --color=auto'
alias ip='ip -color=auto'
""")

source("npm-aliases", "NPM shortcuts: nrun, ntest, nbuild, nstart", """\
alias nrun='npm run'
alias ntest='npm test'
alias nbuild='npm run build'
alias nstart='npm start'
alias nlint='npm run lint'
alias nfix='npm run lint:fix'
alias ndev='npm run dev'
alias ni='npm install'
alias nui='npm uninstall'
alias nug='npm update -g'
alias nls='npm list --depth=0'
alias nout='npm outdated'
alias naud='npm audit'
alias nfix='npm audit fix'
""")

source("cargo-aliases", "Cargo shortcuts: cb, cr, ct, cb", """\
alias cb='cargo build'
alias cbr='cargo build --release'
alias cr='cargo run'
alias crr='cargo run --release'
alias ct='cargo test'
alias ck='cargo check'
alias cl='cargo clippy'
alias cf='cargo fmt'
alias ca='cargo add'
alias crm='cargo remove'
alias cup='cargo update'
alias cin='cargo init'
alias cnew='cargo new'
alias cdoc='cargo doc --open'
alias cout='cargo outdated'
alias ci='cargo install'
""")

source("systemctl-aliases", "Systemd shortcuts: sstart, sstop, srestart, sstatus", """\
alias sstart='sudo systemctl start'
alias sstop='sudo systemctl stop'
alias srestart='sudo systemctl restart'
alias sstatus='systemctl status'
alias senable='sudo systemctl enable'
alias sdisable='sudo systemctl disable'
alias sreload='sudo systemctl daemon-reload'
alias slist='systemctl list-units --type=service'
alias sfail='systemctl --failed'
alias su='systemctl --user'
alias sus='systemctl --user status'
alias suly='systemctl --user list-units'
alias jlog='journalctl -xe -n 50'
alias jlogf='journalctl -xef'
alias jlogu='journalctl -xe -u'
""")

source("kubectl-aliases", "Kubernetes aliases: k, kg, kd, ksys, kall", """\
alias k='kubectl'
alias kg='kubectl get'
alias kgp='kubectl get pods'
alias kgs='kubectl get svc'
alias kgd='kubectl get deploy'
alias kgn='kubectl get nodes'
alias kd='kubectl describe'
alias kdp='kubectl describe pod'
alias kds='kubectl describe svc'
alias kl='kubectl logs'
alias klf='kubectl logs -f'
alias kex='kubectl exec -it'
alias kpf='kubectl port-forward'
alias kall='kubectl get all --all-namespaces'
alias ksys='kubectl --namespace=kube-system'
alias kctx='kubectl config current-context'
alias kns='kubectl config set-context --current --namespace'
alias ktop='kubectl top pods'
""")

source("tmux-aliases", "Tmux session management aliases", """\
alias tm='tmux'
alias tml='tmux list-sessions'
alias tmn='tmux new-session -s'
alias tma='tmux attach -t'
alias tmk='tmux kill-session -t'
alias tmks='tmux kill-server'
alias tmr='tmux rename-session -t'
alias tmw='tmux list-windows'
alias tms='tmux split-window -h'
alias tmv='tmux split-window -v'
alias tmcolor='tmux set -g status-bg colour235'
""")

source("vim-aliases", "Vim/nvim shortcuts", """\
alias v='vim'
alias nv='nvim'
alias vi='vim'
alias vimrc='vim ~/.vimrc'
alias nvrc='nvim ~/.config/nvim/init.lua'
alias vimdiff='vim -d'
alias nvdiff='nvim -d'
alias vimup='vim +PlugUpgrade +PlugUpdate +qa'
alias nvup='nvim +Lazy! sync +qa'
""")

source("python-aliases", "Python aliases: py, py3, venv, pip shortcuts", """\
alias py='python3'
alias py2='python2'
alias venv='python3 -m venv'
alias act='source ./venv/bin/activate'
alias act2='source ./.venv/bin/activate 2>/dev/null || source ./venv/bin/activate'
alias deact='deactivate'
alias pip='python3 -m pip'
alias pipi='python3 -m pip install'
alias pipir='python3 -m pip install -r requirements.txt'
alias pipu='python3 -m pip install --upgrade'
alias pipun='python3 -m pip uninstall'
alias pipl='python3 -m pip list'
alias pipf='python3 -m pip freeze'
alias pyc='python3 -c'
alias pym='python3 -m'
alias pyc='find . -name "*.pyc" -delete && find . -name "__pycache__" -type d -delete'
alias black='python3 -m black'
alias flake='python3 -m flake8'
alias mypy='python3 -m mypy'
alias pytest='python3 -m pytest'
""")

source("rust-aliases", "Rust tool aliases: rustc, rustup, cargo extras", """\
alias ru='rustup'
alias rup='rustup update'
alias rus='rustup show'
alias rut='rustup toolchain list'
alias rutn='rustup toolchain install nightly'
alias run='rustup override set nightly'
alias rustcver='rustc --version'
alias rdoc='rustup doc'
alias rr='rustup run'
alias rcom='rustup component add'
""")

source("go-aliases", "Go language aliases: gob, gor, got, gofmt", """\
alias gob='go build'
alias gor='go run'
alias got='go test'
alias gom='go mod'
alias gomi='go mod init'
alias gomt='go mod tidy'
alias gov='go vet'
alias gof='gofmt -s -w'
alias gol='golint'
alias gog='go get'
alias goi='go install'
alias goc='go clean'
alias gogen='go generate'
alias gocheck='go vet ./... && go test ./...'
alias goall='go build ./... && go test ./...'
""")

source("node-aliases", "Node.js aliases: node, npx, nodemon", """\
alias nodev='node --version'
alias npx='npx'
alias nn='nodemon'
alias nvmal='nvm alias default'
alias nvmuse='nvm use'
alias nvmls='nvm ls'
alias nvmin='nvm install'
alias yarn='yarn'
alias ya='yarn add'
alias yad='yarn add --dev'
alias yrm='yarn remove'
alias yrun='yarn run'
alias ytest='yarn test'
alias ybuild='yarn build'
""")

source("net-aliases", "Network diagnostic aliases: ping, curl, wget, ss", """\
alias myip='curl -s ifconfig.me 2>/dev/null || echo no internet'
alias myipl='ip -4 addr show | grep -oP "(?<=inet )\d+\.\d+\.\d+\.\d+"'
alias pingg='ping 8.8.8.8'
alias p='ping -c 4'
alias fastping='ping -c 100 -s 0'
alias ports='ss -tulanp'
alias listening='ss -tulnp'
alias conn='ss -tup'
alias netstat='ss -tulanp'
alias dns='cat /etc/resolv.conf'
alias flushed='sudo resolvectl flush-caches 2>/dev/null || sudo systemd-resolve --flush-caches 2>/dev/null || true'
alias trace='mtr -t'
alias tracer='traceroute'
alias speed='curl -s https://raw.githubusercontent.com/sivel/speedtest-cli/master/speedtest.py | python3 -'
alias wget='wget -c'
alias curlt='curl -w "\nHTTP %{http_code} | %{time_total}s | %{size_download} bytes\n" -o /dev/null -s'
""")

source("archive-aliases", "Archive/extract aliases: tarball, unzip, extract", """\
alias tarball='tar -cvzf'
alias untar='tar -xvzf'
alias tarlist='tar -tvzf'
alias tarbz='tar -cvjf'
alias untarbz='tar -xvjf'
alias unzip='unzip -o'
alias unrar='unrar x'
alias un7z='7z x'
alias extract='() { if [ -f "$1" ]; then case "$1" in *.tar.gz|*.tgz) tar -xvzf "$1" ;; *.tar.bz2|*.tbz2) tar -xvjf "$1" ;; *.tar.xz) tar -xvJf "$1" ;; *.tar.zst) tar --zstd -xvf "$1" ;; *.tar) tar -xvf "$1" ;; *.gz) gunzip "$1" ;; *.bz2) bunzip2 "$1" ;; *.xz) unxz "$1" ;; *.zst) unzstd "$1" ;; *.zip) unzip "$1" ;; *.rar) unrar x "$1" ;; *.7z) 7z x "$1" ;; *) echo "unknown: $1" ;; esac; else echo "not found: $1"; fi; }'
alias compress='() { tar -cvzf "$1.tar.gz" "${@:2}"; }'
alias compressbz='() { tar -cvjf "$1.tar.bz2" "${@:2}"; }'
""")

source("media-aliases", "Media aliases: ffmpeg, yt-dlp shortcuts", """\
alias yt='yt-dlp'
alias yta='yt-dlp -x --audio-format mp3'
alias ytv='yt-dlp -f bestvideo+bestaudio'
alias ytpl='yt-dlp --yes-playlist'
alias ff='ffmpeg -hide_banner'
alias ffp='ffprobe -hide_banner'
alias gif='ffmpeg -f image2 -framerate 10 -pattern_type glob -i "*.png" output.gif'
alias screen='import -window root screenshot.png'
alias screenselect='import screenshot.png'
alias rec='ffmpeg -f x11grab -video_size 1920x1080 -i :0.0 -c:v libx264 output.mp4'
alias convertimg='convert'
alias magick='magick'
alias resize='mogrify -resize'
alias optimize='pngquant --ext .png --force 256'
alias vc='ffmpeg -i'
alias vcompress='ffmpeg -i "$1" -vcodec libx265 -crf 28 -acodec aac -b:a 128k'
""")

source("dev-aliases", "General dev tools: make, cmake, gcc, gdb", """\
alias makej='make -j$(nproc)'
alias cm='cmake -B build -DCMAKE_BUILD_TYPE=Release'
alias cmb='cmake --build build -j$(nproc)'
alias gdb='gdb -q'
alias gdbt='gdb -tui'
alias val='valgrind --leak-check=full'
alias strace='strace -f'
alias ldd='ldd -r'
alias obj='objdump -d'
alias asm='objdump -d -M intel'
alias size='size'
alias nm='nm -C'
alias strings='strings -a'
alias strip='strip -s'
alias gcc='gcc -Wall -Wextra'
alias gccc='gcc -Wall -Wextra -Werror -pedantic'
alias gccopt='gcc -O2 -Wall -Wextra'
alias cpp='g++ -Wall -Wextra -std=c++17'
alias clang='clang -Wall -Wextra'
alias tidy='clang-tidy'
alias fmt='clang-format -i'
""")

source("ripgrep-aliases", "ripgrep aliases: rg, rgi, rgf", """\
alias rg='rg -S'
alias rgi='rg -Si'
alias rgf='rg -l'
alias rgpy='rg --type py'
alias rgrs='rg --type rs'
alias rgjs='rg --type js'
alias rgc='rg --type c'
alias rgcpp='rg --type cpp'
alias rgmd='rg --type md'
alias rgjson='rg --type json'
alias rgyaml='rg --type yaml'
alias rghtml='rg --type html'
alias rgcss='rg --type css'
alias rgsh='rg --type sh'
alias rgvim='rg -S "TODO|FIXME|HACK|XXX"'
alias rghid='rg --hidden'
alias rgn='rg -S -n'
""")

source("fd-aliases", "fd (find alternative) aliases: fd, fdi, fdh", """\
alias fd='fd -H'
alias fdi='fd -HI'
alias fdh='fd -H -t d'
alias fdf='fd -H -t f'
alias fdl='fd -H -t l'
alias fds='fd -H -t f -x stat'
alias fdd='fd -H --max-depth 1 -t d'
alias fdem='fd -H -e eml'
alias fdmd='fd -H -e md'
alias fdrs='fd -H -e rs'
alias fdpy='fd -H -e py'
alias fdjs='fd -H -e js'
alias fdjson='fd -H -e json'
alias fdyaml='fd -H -e yaml -e yml'
alias fdtoml='fd -H -e toml'
alias fdgo='fd -H -e go'
alias fdc='fd -H -e c -e h'
alias fdcpp='fd -H -e cpp -e hpp -e cc -e hh'
alias fdrs='fd -H -e rs -e rlib'
""")

source("bat-aliases", "bat (cat alternative) aliases: bat, batp, batdiff", """\
alias bat='bat --theme=Dracula'
alias batp='bat --plain'
alias batln='bat --number'
alias batdiff='bat --diff'
alias batl='bat -l'
alias bathelp='bat --help | bat -l help'
alias manbat='() { man "$1" | bat -l man; }'
alias batg='bat --theme=Dracula --paging=never -l'
alias batcsv='bat -l csv --paging=never'
alias batjson='bat -l json --paging=never'
alias batyaml='bat -l yaml --paging=never'
alias batxml='bat -l xml --paging=never'
alias batmd='bat -l markdown --paging=never'
alias batrs='bat -l rust --paging=never'
alias batpy='bat -l python --paging=never'
alias batjs='bat -l javascript --paging=never'
alias batgo='bat -l go --paging=never'
alias batc='bat -l c --paging=never'
alias batcpp='bat -l cpp --paging=never'
alias batconf='bat -l ini --paging=never'
""")

source("tmuxifier", "Tmux session manager functions", """\
tmux-session() {
  local dir="${1:-.}"
  local name="${2:-$(basename "$(realpath "$dir")")}"
  tmux new-session -d -s "$name" -c "$dir" 2>/dev/null || true
  tmux send-keys -t "$name" "cd $dir" Enter
  if tmux has-session -t "$name" 2>/dev/null; then
    tmux attach-session -t "$name"
  fi
}
tmux-vsplit() {
  local dir="${1:-.}"
  tmux split-window -h -c "$dir"
}
tmux-hsplit() {
  local dir="${1:-.}"
  tmux split-window -v -c "$dir"
}
tmux-kill-all() {
  tmux list-sessions -F '#{session_name}' | while read -r s; do
    tmux kill-session -t "$s"
  done
}
""")

source("finder", "fzf-based interactive finders", """\
f() { fzf --preview 'bat --style=numbers --color=always --line-range :500 {}' }
fkill() { ps aux | fzf -m | awk '{print $2}' | xargs kill -9 2>/dev/null || true }
fcd() { cd "$(fd -t d | fzf --preview 'ls -la {}')" || true }
fbr() { git branch -a | fzf | tr -d ' *' | xargs git checkout }
fshow() { git log --graph --color=always --oneline --all | fzf --ansi --preview 'echo {} | grep -o "[a-f0-9]\\+" | xargs git show --stat --color=always' | grep -o "[a-f0-9]\\+" | xargs git show }
fman() { man -k . | fzf | awk '{print $1}' | xargs man }
fdocker() { docker ps -a | fzf | awk '{print $NF}' }
fport() { ss -tulanp | fzf }
""")

source("extract-utils", "Universal extraction and compression tools", """\
extract() {
  if [ -f "$1" ]; then
    case "$1" in
      *.tar.gz|*.tgz) tar -xvzf "$1" ;;
      *.tar.bz2|*.tbz2) tar -xvjf "$1" ;;
      *.tar.xz|*.txz) tar -xvJf "$1" ;;
      *.tar.zst) tar --zstd -xvf "$1" ;;
      *.tar) tar -xvf "$1" ;;
      *.gz) gunzip "$1" ;;
      *.bz2) bunzip2 "$1" ;;
      *.xz) unxz "$1" ;;
      *.zst) unzstd "$1" ;;
      *.zip) unzip "$1" ;;
      *.rar) unrar x "$1" ;;
      *.7z) 7z x "$1" ;;
      *.lz4) lz4 -d "$1" ;;
      *.sz) snzip -d "$1" ;;
      *.Z) uncompress "$1" ;;
      *) echo "unknown format: $1" ;;
    esac
  else
    echo "not found: $1"
  fi
}
compress() {
  local name="$1" ext="${2:-tar.gz}" ; shift
  case "$ext" in
    tar.gz|tgz) tar -cvzf "$name.$ext" "$@" ;;
    tar.bz2|tbz2) tar -cvjf "$name.$ext" "$@" ;;
    tar.xz|txz) tar -cvJf "$name.$ext" "$@" ;;
    tar.zst) tar --zstd -cvf "$name.$ext" "$@" ;;
    zip) zip -r "$name.zip" "$@" ;;
    *) echo "unsupported: $ext" ;;
  esac
}
""")

source("mkcd", "Create directory and cd into it in one command", """\
mkcd() { mkdir -p "$1" && cd "$1" || true; }
mkcp() { mkdir -p "$(dirname "$2")" && cp "$1" "$2"; }
mkmv() { mkdir -p "$(dirname "$2")" && mv "$1" "$2"; }
mkln() { mkdir -p "$(dirname "$2")" && ln -s "$1" "$2"; }
""")

source("timer", "Simple timer and stopwatch functions", """\
timer() {
  local start end elapsed
  start=$(date +%s)
  "$@"
  end=$(date +%s)
  elapsed=$((end - start))
  echo "took ${elapsed}s"
}
stopwatch() {
  local start=$SECONDS
  while true; do
    local now=$SECONDS
    local h=$((now / 3600))
    local m=$(( (now % 3600) / 60 ))
    local s=$((now % 60))
    printf "\r%02d:%02d:%02d" "$h" "$m" "$s"
    sleep 0.5
  done
}
countdown() {
  local s="${1:-10}"
  while [ "$s" -gt 0 ]; do
    printf "\r%02d" "$s"
    sleep 1
    s=$((s - 1))
  done
  printf "\rDONE! \n"
}
""")

source("path-utils", "PATH and environment variable helpers", """\
path() { echo "$PATH" | tr ':' '\n' | nl; }
path-add() { export PATH="$1:$PATH"; }
path-rm() { export PATH=$(echo "$PATH" | tr ':' '\n' | grep -v "$1" | tr '\n' ':') }
ldpath() { echo "$LD_LIBRARY_PATH" | tr ':' '\n' | nl; }
envline() { env | sort | nl; }
alias envs='env | sort'
alias findcmd='type -a'
""")

source("pskiller", "Process management utilities", """\
pskill() { kill -9 "$(pgrep -f "$1")" 2>/dev/null || echo "no process matching: $1"; }
psnice() { renice -n "${2:-10}" "$(pgrep -f "$1")"; }
pswatch() { watch -n 1 "ps aux | grep '$1'"; }
psmem() { ps aux --sort=-%mem | head -20; }
pscpu() { ps aux --sort=-%cpu | head -20; }
pstre() { pstree -p "$USER" | head -50; }
""")

source("webutils", "Web utility functions", """\
headers() { curl -sI "$1"; }
http() { curl -s -o /dev/null -w "HTTP %{http_code} | %{time_total}s | %{size_download}b\n" "$1"; }
downslow() { curl --limit-rate "$1" -O "$2"; }
srv() { python3 -m http.server "${1:-8000}"; }
srv6() { python3 -m http.server "${1:-8000}" --bind ::; }
share() { curl -s -F "file=@$1" https://0x0.st; }
shorten() { curl -s -F "url=$1" https://ttm.sh; }
alias wttr='curl -s "wttr.in/?0TQ" | head -n -2'
alias moon='curl -s "wttr.in/Moon" | head -n -2'
alias weather='() { curl -s "wttr.in/${1:-}" | head -n -2; }'
""")

source("git-extras", "Extended Git utilities and workflows", """\
gundo() { git reset --soft HEAD~1; }
gredo() { git commit -c ORIG_HEAD; }
gfix() { git commit --fixup "$1"; }
gsquash() { git rebase -i --autosquash "$1"; }
gpr() { git pull --rebase; }
gpurge() { git branch --merged | grep -v '\*\|main\|master' | xargs -r git branch -d; }
gclean() { git clean -fd; }
gcl() { git clone --recurse-submodules "$1"; }
gtag() { git tag -a "$1" -m "$2"; }
gcontrib() { git shortlog -sn; }
gblam() { git blame -w "$1"; }
gsearch() { git log --all -G"$1" --source; }
alias gwho='git shortlog -sne'
""")

source("docker-extras", "Docker advanced utilities", """\
dclean() { docker system prune -af --volumes; }
drmall() { docker rm -f $(docker ps -aq) 2>/dev/null || true; }
drmiall() { docker rmi -f $(docker images -q) 2>/dev/null || true; }
dstats() { docker stats --no-stream; }
dtop() { docker top "$1"; }
dinspect() { docker inspect "$1" | bat -l json; }
dsh() { docker exec -it "$1" /bin/sh; }
dbash() { docker exec -it "$1" /bin/bash; }
dnet() { docker network ls; }
dvol() { docker volume ls; }
dcompose-up() { docker compose up -d; }
dcompose-down() { docker compose down -v; }
dlogq() { docker logs -n 20 "$1"; }
alias dcup='docker compose up -d'
alias dcdown='docker compose down'
alias dcbuild='docker compose build'
alias dcrestart='docker compose restart'
""")

source("ripgrep-extras", "Advanced ripgrep usage", """\
rgfind() { rg -l "$1" --sort path; }
rgcount() { rg -c "$1" | sort -t: -k2 -rn; }
rgctx() { rg -C "${2:-3}" "$1"; }
rgjson() { rg "$1" --type json -C 2; }
rglog() { rg "$1" --type log; }
rgvim() { rg 'TODO|FIXME|HACK|XXX|OPTIMIZE' --type-add 'all:*' -T binary; }
rgr() { rg "$1" -r "$2"; }
rgstats() { rg -c "" | awk -F: '{s+=$2} END {print s " matches"}'; }
rghid() { rg --no-ignore --hidden "$1"; }
rgm() { rg "$1" --type-add 'web:*.{html,css,js,ts,jsx,tsx}' -t web; }
""")

source("fd-extras", "Advanced fd usage", """\
fdbig() { fd -t f -S +"${1:-10M}" --exec ls -lh; }
fdempty() { fd -t f -s 0b; }
fdrecent() { fd -t f --changed-within "${1:-24h}"; }
fdold() { fd -t f --changed-before "${1:-30d}"; }
fddup() { fd -t f | xargs -I{} basename {} | sort | uniq -d; }
fddirs() { fd -t d -d "${1:-3}"; }
fdsize() { fd -t f -S +"${1:-1M}" -x du -sh | sort -rh; }
fdext() { fd -t f -e "$1" -x echo; }
fdtype() { fd -t "$1"; }
""")

source("calc", "Calculator and math functions", """\
calc() { echo "scale=2; $*" | bc -l; }
calchex() { printf '0x%X\\n' "$1"; }
calcoct() { printf '0%o\\n' "$1"; }
calcbin() { echo "obase=2;$1" | bc; }
calcfloat() { python3 -c "print($*)"; }
csum() { python3 -c "print(sum(float(x) for x in '$*'.split()))"; }
cavg() { python3 -c "vals=[float(x) for x in '$*'.split()]; print(sum(vals)/len(vals))"; }
alias cpi='python3 -c "import math; print(math.pi)"'
alias ce='python3 -c "import math; print(math.e)"'
alias crand='python3 -c "import random; print(random.random())"'
""")

source("cheat", "Quick reference sheets", """\
alias cheat-tar='tar --help | bat -l man'
alias cheat-find='find --help 2>&1 | bat -l man'
alias cheat-git='git help --all | bat -l man'
alias cheat-curl='curl --help | bat -l man'
alias cheat-awk='python3 -c "print(\"awk '{print \\\$1}' file\")"'
alias cheat-sed='python3 -c "print(\"sed 's/old/new/g' file\")"'
alias cheat-sort='sort --help | bat -l man'
alias cheat-uniq='uniq --help | bat -l man'
alias cheat-xargs='xargs --help | bat -l man'
alias cheat-ffmpeg='ffmpeg --help | bat -l man'
alias cheat-ytdlp='yt-dlp --help | bat -l man'
""")

source("misc-aliases", "Miscellaneous useful aliases", """\
alias reload='exec "$SHELL" -l'
alias please='sudo $(fc -ln -1)'
alias fucking='sudo $(fc -ln -1)'
alias fucking='sudo !!'
alias root='sudo -i'
alias suroot='sudo su -'
alias mkdir='mkdir -p'
alias ll='ls -lhF'
alias la='ls -A'
alias l='ls -CF'
alias tree='tree -C'
alias treed='tree -C -d'
alias df='df -h'
alias du='du -h'
alias dus='du -sh * | sort -h'
alias duh='du -sh ./*/ | sort -h'
alias free='free -h'
alias disk='df -h .'
alias usage='du -sh'
alias mount='mount | column -t'
alias ps='ps auxf'
alias pst='ps auxf --sort=-%cpu | head -20'
alias top='htop'
alias n='nano'
alias se='sudo -e'
alias fuck='sudo $(history -p \\!\\!)'
alias down='systemctl poweroff -i'
alias reboot='systemctl reboot'
alias suspend='systemctl suspend'
alias hibernate='systemctl hibernate'
alias c='clear'
alias x='exit'
alias q='exit'
alias h='history'
alias j='jobs -l'
alias ts='timestamp'
alias timestamp='date +%Y%m%d-%H%M%S'
alias now='date +"%Y-%m-%d %H:%M:%S"'
alias today='date +"%A, %B %d, %Y"'
alias week='date +%V'
alias epoch='date +%s'
alias epochms='date +%s%3N'
alias iso='date -I'
alias isodt='date -Iseconds'
alias utc='date -u'
alias cal='cal -3'
alias weekn='date +%V'
alias beep='echo -e "\\a"'
alias alert='notify-send --urgency=low -i "$([ $? = 0 ] && echo terminal || echo error)" "$(history|tail -n1|sed -e "s/^[0-9]\\+//;s/[;&|] alert$//")"'
""")

source("xdg-aliases", "XDG base directory shortcuts", """\
alias cdf='cd "$XDG_CONFIG_HOME"'
alias cdd='cd "$XDG_DATA_HOME"'
alias cdc='cd "$XDG_CACHE_HOME"'
alias cdconf='cd "$HOME/.config"'
alias cdloc='cd "$HOME/.local/share"'
alias cdbin='cd "$HOME/.local/bin"'
alias cdssh='cd "$HOME/.ssh"'
alias cddown='cd "$HOME/Downloads"'
alias cddoc='cd "$HOME/Documents"'
alias cddesk='cd "$HOME/Desktop"'
alias cdpic='cd "$HOME/Pictures"'
alias cdvid='cd "$HOME/Videos"'
alias cdmus='cd "$HOME/Music"'
alias cdtmp='cd /tmp'
alias cdrepo='cd /tmp'
""")

source("lang-aliases", "Programming language REPLs and tools", """\
alias rb='irb'
alias rbpry='pry'
alias py='python3'
alias pyi='python3 -c "import sys; print(sys.version)"'
alias ipy='ipython'
alias julia='julia'
alias jl='julia'
alias lua='lua'
alias luai='lua -i'
alias pl='perl -e'
alias php='php -a'
alias scala='scala'
alias clj='clojure'
alias racket='racket'
alias elixir='iex'
alias erl='erl'
alias ghci='ghci'
alias ghcup='ghcup'
alias haskell='ghci'
alias deno='deno'
alias bun='bun'
alias denor='deno run'
alias bunr='bun run'
""")

source("text-utils", "Text processing utilities", """\
alias jq='jq -C'
alias jqr='jq -r'
alias yq='yq'
alias yqr='yq -r'
alias toml2json='python3 -c "import sys,tomllib,json; json.dump(tomllib.load(sys.stdin),sys.stdout)"'
alias json2toml='python3 -c "import sys,toml,json; toml.dump(json.load(sys.stdin),sys.stdout)"'
alias csvview='python3 -c "import sys,csv; [print(\"|\".join(r)) for r in csv.reader(sys.stdin)]"'
alias csv2json='python3 -c "import sys,csv,json; w=csv.DictReader(sys.stdin); json.dump(list(w),sys.stdout,indent=2)"'
alias json2csv='python3 -c "import sys,csv,json; d=json.load(sys.stdin); w=csv.DictWriter(sys.stdout,fieldnames=d[0].keys()); w.writeheader(); w.writerows(d)"'
alias pretty='python3 -m json.tool'
alias flatten='tr -d "\\n"'
alias bytes='xxd'
alias hex='xxd | less'
alias dec='python3 -c "print(int(\"$1\", 16))"'
alias reverse='rev'
alias rot13='tr "A-Za-z" "N-ZA-Mn-za-m"'
alias base64e='base64'
alias base64d='base64 -d'
alias urlencode='python3 -c "import sys,urllib.parse; print(urllib.parse.quote(sys.stdin.read()))"'
alias urldecode='python3 -c "import sys,urllib.parse; print(urllib.parse.unquote(sys.stdin.read()))"'
""")

# ── COMPILE PACKAGES — small C programs ──────────────────────────

compile_c("hexdump", "Hex-dump stdin to stdout", """\
#include <stdio.h>
#include <ctype.h>
int main() {
    unsigned char buf[16]; size_t n; unsigned long offset = 0;
    while ((n = fread(buf, 1, 16, stdin)) > 0) {
        printf("%08lx  ", offset);
        for (size_t i = 0; i < 16; i++) {
            if (i < n) printf("%02x ", buf[i]); else printf("   ");
            if (i == 7) printf(" ");
        }
        printf(" |");
        for (size_t i = 0; i < n; i++) putchar(isprint(buf[i]) ? buf[i] : '.');
        printf("|\\n");
        offset += n;
    }
    return 0;
}
""")

compile_c("ticker", "Real-time system stats in your terminal", """\
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
int main() {
    char line[256]; FILE *fp;
    for (;;) {
        printf("\\033[2J\\033[H");
        printf("=== ticker ===\\n\\n");
        fp = fopen("/proc/loadavg", "r");
        if (fp) { fgets(line, sizeof(line), fp); fclose(fp); printf("load: %s", line); }
        fp = fopen("/proc/meminfo", "r");
        if (fp) {
            for (int i = 0; i < 3 && fgets(line, sizeof(line), fp); i++) printf("%s", line);
            fclose(fp);
        }
        printf("\\nctrl-c to exit\\n");
        sleep(1);
    }
    return 0;
}
""")

compile_c("true", "Do-nothing C program that returns 0", """\
int main() { return 0; }
""")

compile_c("yes", "Repeatedly output a line with specified string or y", """\
#include <stdio.h>
int main(int argc, char **argv) {
    const char *s = argc > 1 ? argv[1] : "y";
    for (;;) puts(s);
    return 0;
}
""")

compile_c("no", "Repeatedly output n", """\
#include <stdio.h>
int main() { for (;;) puts("n"); return 0; }
""")

compile_c("tac", "Reverse lines of input", """\
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main() {
    char **lines = NULL; size_t cap = 0, len = 0; char *buf = NULL; size_t n = 0;
    while (getline(&buf, &n, stdin) > 0) {
        if (len >= cap) { cap = cap ? cap * 2 : 256; lines = realloc(lines, cap * sizeof(char*)); }
        lines[len++] = strdup(buf);
    }
    free(buf);
    while (len > 0) fputs(lines[--len], stdout);
    for (size_t i = 0; i < len; i++) free(lines[i]);
    free(lines);
    return 0;
}
""")

compile_c("base64", "Base64 encode/decode stdin", """\
#include <stdio.h>
#include <string.h>
static const char b64[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
int main(int argc, char **argv) {
    int decode = argc > 1 && strcmp(argv[1], "-d") == 0;
    if (decode) {
        unsigned char buf[4], out[3]; int c, i = 0, bits = 0;
        while ((c = getchar()) != EOF) {
            if (c == '=') break;
            const char *p = strchr(b64, c); if (!p) continue;
            buf[i++] = p - b64; bits += 6;
            if (bits >= 24) {
                out[0] = (buf[0] << 2) | (buf[1] >> 4);
                out[1] = (buf[1] << 4) | (buf[2] >> 2);
                out[2] = (buf[2] << 6) | buf[3];
                fwrite(out, 1, 3, stdout); i = 0; bits = 0;
            }
        }
    } else {
        unsigned char buf[3]; size_t n;
        while ((n = fread(buf, 1, 3, stdin)) > 0) {
            putchar(b64[buf[0] >> 2]);
            putchar(b64[((buf[0] & 3) << 4) | (n > 1 ? (buf[1] >> 4) : 0)]);
            putchar(n > 1 ? b64[((buf[1] & 15) << 2) | (n > 2 ? (buf[2] >> 6) : 0)] : '=');
            putchar(n > 2 ? b64[buf[2] & 63] : '=');
        }
    }
    return 0;
}
""")

compile_c("factor", "Factor integers", """\
#include <stdio.h>
#include <stdlib.h>
int main(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        long n = atol(argv[i]); printf("%ld:", n);
        for (long p = 2; p * p <= n; p++) while (n % p == 0) { printf(" %ld", p); n /= p; }
        if (n > 1) printf(" %ld", n);
        printf("\\n");
    }
    return 0;
}
""")

compile_c("primes", "Generate prime numbers up to N", """\
#include <stdio.h>
#include <stdlib.h>
int main(int argc, char **argv) {
    long limit = argc > 1 ? atol(argv[1]) : 1000;
    char *sieve = calloc(limit + 1, 1); if (!sieve) return 1;
    for (long p = 2; p * p <= limit; p++)
        if (!sieve[p]) for (long m = p * p; m <= limit; m += p) sieve[m] = 1;
    for (long p = 2; p <= limit; p++) if (!sieve[p]) printf("%ld\\n", p);
    free(sieve);
    return 0;
}
""")

compile_c("seq", "Print sequences of numbers", """\
#include <stdio.h>
#include <stdlib.h>
int main(int argc, char **argv) {
    double start = 1, step = 1, end = 1;
    if (argc == 2) end = atof(argv[1]);
    else if (argc == 3) { start = atof(argv[1]); end = atof(argv[2]); }
    else if (argc >= 4) { start = atof(argv[1]); step = atof(argv[2]); end = atof(argv[3]); }
    if (step == 0) return 1;
    if (step > 0) for (double i = start; i <= end; i += step) printf("%g\\n", i);
    else for (double i = start; i >= end; i += step) printf("%g\\n", i);
    return 0;
}
""")

compile_c("sha256", "Compute SHA-256 hash of stdin", """\
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#define ROTR(x,n) (((x)>>(n))|((x)<<(32-(n))))
#define CH(x,y,z) (((x)&(y))^(~(x)&(z)))
#define MAJ(x,y,z) (((x)&(y))^((x)&(z))^((y)&(z)))
#define EP0(x) (ROTR(x,2)^ROTR(x,13)^ROTR(x,22))
#define EP1(x) (ROTR(x,6)^ROTR(x,11)^ROTR(x,25))
#define SIG0(x) (ROTR(x,7)^ROTR(x,18)^((x)>>3))
#define SIG1(x) (ROTR(x,17)^ROTR(x,19)^((x)>>10))
int main() {
    uint32_t h[8] = {0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19};
    uint32_t k[64] = {0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2};
    unsigned char buf[64]; size_t n; uint64_t bits = 0;
    uint32_t w[64]; int chunk = 0;
    while ((n = fread(buf, 1, chunk ? 64 : 64, stdin)) > 0 || !chunk) {
        if (!chunk) { chunk = 1; continue; }
        bits += n * 8; int pad = n < 56;
        if (pad) {
            buf[n++] = 0x80;
            while (n < 56) buf[n++] = 0;
            for (int i = 0; i < 8; i++) buf[n++] = (bits >> (56 - i * 8)) & 0xff;
        } else {
            if (n < 64) { buf[n++] = 0x80; while (n < 64) buf[n++] = 0; }
        }
        for (int t = 0; t < 16; t++) w[t] = (buf[t*4]<<24)|(buf[t*4+1]<<16)|(buf[t*4+2]<<8)|buf[t*4+3];
        for (int t = 16; t < 64; t++) w[t] = SIG1(w[t-2])+w[t-7]+SIG0(w[t-15])+w[t-16];
        uint32_t a=h[0],b=h[1],c=h[2],d=h[3],e=h[4],f=h[5],g=h[6],hh=h[7];
        for (int t = 0; t < 64; t++) {
            uint32_t t1 = hh + EP1(e) + CH(e,f,g) + k[t] + w[t];
            uint32_t t2 = EP0(a) + MAJ(a,b,c);
            hh = g; g = f; f = e; e = d + t1; d = c; c = b; b = a; a = t1 + t2;
        }
        h[0]+=a; h[1]+=b; h[2]+=c; h[3]+=d; h[4]+=e; h[5]+=f; h[6]+=g; h[7]+=hh;
        if (pad) break;
    }
    for (int i = 0; i < 8; i++) printf("%08x", h[i]);
    printf("\\n");
    return 0;
}
""")

compile_c("wl", "Word and line count", """\
#include <stdio.h>
int main() {
    long words = 0, lines = 0, chars = 0; int c, in = 0;
    while ((c = getchar()) != EOF) {
        chars++; if (c == '\\n') lines++;
        if (c == ' ' || c == '\\n' || c == '\\t') in = 0;
        else if (!in) { in = 1; words++; }
    }
    printf("%ld %ld %ld\\n", lines, words, chars);
    return 0;
}
""")

compile_c("uniquify", "Filter unique lines (like uniq)", """\
#include <stdio.h>
#include <string.h>
int main() {
    char *prev = NULL, *buf = NULL; size_t n = 0;
    while (getline(&buf, &n, stdin) > 0) {
        if (!prev || strcmp(buf, prev) != 0) { fputs(buf, stdout); free(prev); prev = strdup(buf); }
    }
    free(prev); free(buf);
    return 0;
}
""")

compile_c("expandtab", "Convert tabs to spaces", """\
#include <stdio.h>
int main(int argc, char **argv) {
    int ts = argc > 1 ? atoi(argv[1]) : 8; if (ts < 1) ts = 8;
    int c, col = 0;
    while ((c = getchar()) != EOF) {
        if (c == '\\t') { int n = ts - col % ts; while (n--) { putchar(' '); col++; } }
        else { putchar(c); col++; if (c == '\\n') col = 0; }
    }
    return 0;
}
""")

# ── COMPILE RUST PACKAGES ──

compile_rust("cpuid", "Print CPU information from /proc/cpuinfo", """\
fn main() {
    let data = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    for line in data.lines().take(30) {
        if line.contains("model name") || line.contains("cpu cores")
            || line.contains("siblings") || line.contains("cache size")
            || line.contains("cpu MHz") || line.contains("flags")
            || line.contains("vendor_id") || line.contains("cpu family") {
            println!("{}", line.trim());
        }
    }
}
""")

compile_rust("dirs", "Print directory sizes for current dir", """\
use std::fs;
fn main() {
    if let Ok(entries) = fs::read_dir(".") {
        let mut items: Vec<(String, u64)> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().ok().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                let size = dir_size(&e.path());
                Some((name, size))
            })
            .collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));
        for (name, size) in items {
            let s = if size > 1_000_000_000 { format!("{:.2}G", size as f64 / 1e9) }
                    else if size > 1_000_000 { format!("{:.2}M", size as f64 / 1e6) }
                    else if size > 1_000 { format!("{:.2}K", size as f64 / 1e3) }
                    else { format!("{}B", size) };
            println!("{:>8}  {}", s, name);
        }
    }
}
fn dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_dir() { total += dir_size(&entry.path()); }
                else { total += meta.len(); }
            }
        }
    }
    total
}
""")

compile_rust("battery", "Show battery status", """\
fn main() {
    let bat_dir = "/sys/class/power_supply";
    if let Ok(entries) = std::fs::read_dir(bat_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("BAT") && !name.starts_with("bat") { continue; }
            let p = |f: &str| std::fs::read_to_string(format!("{}/{}/{}", bat_dir, name, f))
                .unwrap_or_default().trim().to_string();
            let status = p("status");
            let cap = p("capacity");
            let volt = p("voltage_now");
            let volt_v = volt.parse::<f64>().map(|v| v / 1_000_000.0).unwrap_or(0.0);
            println!("{}: {}  {}%  {:.3}V", name, status, cap, volt_v);
        }
    }
}
""")

compile_rust("uptimers", "Show system uptime nicely", """\
fn main() {
    let data = std::fs::read_to_string("/proc/uptime").unwrap_or_default();
    let secs = data.split_whitespace().next()
        .and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) as u64;
    let d = secs / 86400; let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60; let s = secs % 60;
    if d > 0 { print!("{}d ", d); }
    print!("{:02}:{:02}:{:02}\\n", h, m, s);
}
""")

compile_rust("sensors", "Show temperature sensors (reads /sys/class/thermal)", """\
fn main() {
    let thermal = "/sys/class/thermal";
    if let Ok(entries) = std::fs::read_dir(thermal) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("thermal_zone") { continue; }
            let typ = std::fs::read_to_string(format!("{}/{}/type", thermal, name))
                .unwrap_or_default().trim().to_string();
            let temp = std::fs::read_to_string(format!("{}/{}/temp", thermal, name))
                .unwrap_or_default().trim().to_string();
            if let Ok(millic) = temp.parse::<f64>() {
                println!("{}: {:.1}°C", typ, millic / 1000.0);
            }
        }
    }
}
""")

compile_rust("mounts", "Show mounted filesystems in a table", """\
fn main() {
    let data = std::fs::read_to_string("/proc/mounts").unwrap_or_default();
    for line in data.lines().take(50) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let (dev, mount, fstype) = (parts[0], parts[1], parts[2]);
            if !dev.starts_with("/dev") { continue; }
            println!("{:<20} {:<30} {}", dev, mount, fstype);
        }
    }
}
""")

compile_rust("pwd-rs", "Print working directory with resolved symlinks", """\
fn main() {
    match std::env::current_dir() {
        Ok(p) => println!("{}", p.display()),
        Err(e) => eprintln!("error: {}", e),
    }
}
""")

# ── BUILD PACKAGES — clone from GitHub and compile ────────────────

with open(ASCII_FILE) as f:
    ascii_logo = f.read()

sysfetch_files = [
    {"path": "logo.txt", "content": ascii_logo},
    {"path": "setup.sh", "content": "mkdir -p \"$HOME/.config/fastfetch\" 2>/dev/null || true\ncp \"$UPD_PKG_DIR/logo.txt\" \"$HOME/.config/fastfetch/logo.txt\" 2>/dev/null || true\n"},
]

build_pkg("sysfetch", "Fast system info tool with custom ASCII globe logo",
          "https://github.com/fastfetch-cli/fastfetch", "fastfetch",
          "cmake", "fastfetch-cli / updSH", "2.0.0", sysfetch_files)

build_pkg("ripgrep-build", "ripgrep — recursively searches directories for a regex pattern",
          "https://github.com/BurntSushi/ripgrep", "rg", "cargo",
          "BurntSushi", "14.1.0")

build_pkg("fd-build", "fd — a simple, fast alternative to find",
          "https://github.com/sharkdp/fd", "fd", "cargo",
          "sharkdp", "10.2.0")

build_pkg("bat-build", "bat — a cat(1) clone with wings (syntax highlighting)",
          "https://github.com/sharkdp/bat", "bat", "cargo",
          "sharkdp", "0.24.0")

build_pkg("eza-build", "eza — a modern ls replacement",
          "https://github.com/eza-community/eza", "eza", "cargo",
          "eza-community", "0.20.0")

build_pkg("delta-build", "delta — a syntax-highlighting pager for git, diff, and grep output",
          "https://github.com/dandavison/delta", "delta", "cargo",
          "dandavison", "0.18.0")

build_pkg("zoxide-build", "zoxide — a smarter cd command",
          "https://github.com/ajeetdsouza/zoxide", "zoxide", "cargo",
          "ajeetdsouza", "0.9.6")

build_pkg("tealdeer-build", "tealdeer — fast tldr client in Rust",
          "https://github.com/dbrgn/tealdeer", "tldr", "cargo",
          "dbrgn", "1.7.1")

build_pkg("bottom-build", "bottom — yet another cross-platform graphical process/system monitor",
          "https://github.com/ClementTsang/bottom", "btm", "cargo",
          "ClementTsang", "0.10.2")

build_pkg("bandwhich-build", "bandwhich — terminal bandwidth utilization tool",
          "https://github.com/imsnif/bandwhich", "bandwhich", "cargo",
          "imsnif", "0.22.2")

build_pkg("gping-build", "gping — ping, but with a graph",
          "https://github.com/orf/gping", "gping", "cargo",
          "orf", "1.17.3")

build_pkg("du-dust-build", "du-dust — a more intuitive version of du",
          "https://github.com/bootandy/dust", "dust", "cargo",
          "bootandy", "1.1.1")

build_pkg("procs-build", "procs — a modern replacement for ps",
          "https://github.com/dalance/procs", "procs", "cargo",
          "dalance", "0.14.8")

build_pkg("hyperfine-build", "hyperfine — a command-line benchmarking tool",
          "https://github.com/sharkdp/hyperfine", "hyperfine", "cargo",
          "sharkdp", "1.18.0")

build_pkg("doggo-build", "doggo — a command-line DNS client for humans",
          "https://github.com/mr-karan/doggo", "doggo", "cargo",
          "mr-karan", "1.0.0")

build_pkg("curlie-build", "curlie — the power of curl with the ease of httpie",
          "https://github.com/rs/curlie", "curlie", "cargo",
          "rs", "1.7.2")

build_pkg("fzf-build", "fzf — a general-purpose command-line fuzzy finder",
          "https://github.com/junegunn/fzf", "fzf", "cargo",
          "junegunn", "0.56.0")

build_pkg("hexyl-build", "hexyl — a command-line hex viewer (Rust)",
          "https://github.com/sharkdp/hexyl", "hexyl", "cargo",
          "sharkdp", "0.14.0")

build_pkg("grex-build", "grex — generates regular expressions from test cases",
          "https://github.com/pemistahl/grex", "grex", "cargo",
          "pemistahl", "1.4.5")

# ── WRITE OUTPUTS ─────────────────────────────────────────────────

os.makedirs(os.path.dirname(OUTPUT_RUST), exist_ok=True)
with open(OUTPUT_RUST, "w") as f:
    json.dump(packages, f, indent=2)

os.makedirs(os.path.dirname(OUTPUT_VERCEL), exist_ok=True)
with open(OUTPUT_VERCEL, "w") as f:
    json.dump(packages, f, indent=2)

print(f"Generated {len(packages)} packages")
print(f"  -> {OUTPUT_RUST}")
print(f"  -> {OUTPUT_VERCEL}")
