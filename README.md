# mokey

> [English](README.en.md) | 한국어

키보드로 마우스를 조종하는 오픈소스 도구. [mouseless](https://github.com/milgra/mouseless)의 대안으로, 그리드 줌(숫자 입력 방식) UX와 선택적 Vim 모드를 제공합니다.

- **그리드 줌**: 트리거 키 → 화면이 숫자 격자로 분할 → 번호 입력으로 확대 반복 → Enter/클릭
- **Vim 모드 (기본 OFF)**: `hjkl` 이동, `m` 좌클릭, `,.` 스크롤, `/e/y/v` 드래그, 숫자 반복 횟수
- **첫 사용자 직관성**: 트리거 후 바로 번호를 누르면 되는 구조, Vim 바인딩은 설정에서 켭니다
- **경량**: Rust + egui, 웹뷰 없음, 단일 바이너리

## 지원 플랫폼

| 플랫폼 | 상태 |
| --- | --- |
| Windows 11 (x64) | ✅ 개발 중 (Phase 1) |
| Hyprland (Wayland) | ⏳ Phase 2 |
| KDE Plasma (Wayland) | ⏳ Phase 3 |
| GNOME (Wayland) | ⏳ Phase 3 |

X11, macOS는 계획에 없습니다.

## 빌드

```sh
cargo build --release
```

Windows에서 로우레벨 키/마우스 API(rdev, enigo, windows-sys)를 사용하므로
관리자 권한 없이도 대부분 동작하지만, 일부 앱에서 전역 후킹 제한이 있을 수 있습니다.

## 사용법

1. `mokey` 실행 (트레이에서 상시 대기)
2. `Ctrl+Alt+Space` → 화면에 숫자 격자 표시
3. 목표 셀 번호 입력 → 확대 반복 → `Enter` 클릭, `Backspace` 확대 취소, `Esc` 종료
4. 설정창: `Ctrl+Alt+S`

설정 파일: `%USERPROFILE%\.config\mokey\config.toml`

```toml
[general]
trigger_hotkey = "Ctrl+Alt+Space"
settings_hotkey = "Ctrl+Alt+S"
grid_size = 3
max_depth = 6
auto_click = false
overlay_bg_opacity = 0.45
move_step = 10
move_fast_step = 100
theme = "dark"

[vim]
enabled = false

[custom_themes.dracula-custom]
overlay = "#1E1E2EBB"
grid = "#F5C2E7"
label = "#CDD6F4"
accent = "#CBA6F7"
hint_bg = "#1E1E2EE6"
hint_text = "#A6ADC8"
status = "#F38BA8"
bg = "#181825"
panel = "#1E1E2E"
text = "#CDD6F4"
dark = true
```

테마는 설정창(`Ctrl+Alt+S`)의 Theme 섹션에서 고르거나 만들 수 있습니다. 빌트인: `dark` · `dracula` · `nord` · `light`.

## 아키텍처

- **`mokey-core`**: 플랫폼 무관 로직(격자 계산, 세션 상태, 키 입력 파싱, 설정 스키마)
- **`mokey-backend`**: 플랫폼별 마우스 제어(enigo), 전역 키 후킹(rdev), 모니터/DPI 열거(windows-sys). Wayland에서는 layer-shell/hyprctl/ydotool/KGlobalAccel 계획
- **`mokey-app`**: egui 기반 HUD 오버레이 + 설정창 (eframe)

## 로드맵

- Phase 1: Windows MVP (현재)
- Phase 2: Hyprland 지원
- Phase 3: KDE Plasma + GNOME 지원
- Phase 4: 추가 기능(제스처, 좌표북마크 등)

## 개발 로그

진행 과정과 디버깅 기록은 [DEVELOPMENT_LOG.md](DEVELOPMENT_LOG.md) 참고.

## 라이선스

MIT. 자세한 내용은 [LICENSE](LICENSE) 참고.
