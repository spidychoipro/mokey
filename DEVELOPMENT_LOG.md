# mokey 개발 로그

키보드로 마우스를 조종하는 오픈소스 도구 `mokey`의 개발 과정을 기록한 문서입니다.
언제 어떤 문제가 있었고, 어떻게 해결했는지 정리해 두어 이후 개발에 참고합니다.

## 프로젝트 개요

- **목표**: [mouseless](https://github.com/milgra/mouseless)의 대안. 그리드 줌(숫자 입력) UX + 선택적 Vim 모드.
- **구조**: `mokey-core`(플랫폼 무관 로직) / `mokey-backend`(rdev·enigo·windows-sys) / `mokey-app`(egui HUD)
- **플랫폼**: Windows 11 MVP → Hyprland/KDE/GNOME(Wayland) 계획

## 변경 이력

### 2026-08-06 — 설정 창 UX 정비 + 테마 시스템 도입

**1. 설정 창 닫기 불가 (X 버튼/Alt+F4)**

- 증상: 설정 창이 X/Alt+F4로 닫히지 않고, 처음 열릴 때 작업 표시줄에만 뜸.
- 수정: `close_requested()`를 감지해 `settings_open = false`. 위치는 빌더 `with_position` 대신 창 생성 후 `ViewportCommand::OuterPosition(모니터 중앙)` + `Focus` 커맨드 전송(HUD 검증된 방식)으로 해결.

**2. 설정 토글 즉시 원복 (vim 체크 즉시 해제)**

- 원인: `settings_ui`가 매 프레임 `controller.config`를 복제해 편집 → 포커스 잃을 때마다 원복.
- 수정: `settings_cfg: Option<Config>` 드래프트를 유지. UI는 드래프트만 편집, Save 시 반영 + `Config::save`.

**3. 테마 시스템 (dracula 포함 다중 테마 + 사용자 테마 제작)**

- `mokey-core/src/theme.rs` 신규: `Rgba`(hex 직렬화), `Theme`(overlay/grid/label/accent/hint_bg/hint_text/status/bg/panel/text/dark), 빌트인 `dark`/`dracula`/`nord`/`light`, `Theme::resolve`(custom 우선 → 빌트인 → dark). hex 왕복/리졸브 테스트 포함.
- `Config` 확장: `general.theme`(활성 테마, 기본 "dark") + `custom_themes: BTreeMap<String, Theme>`.
- HUD 렌더링 전면 테마화(`draw_hud`), 설정창/패널/텍스트 색은 `apply_visuals`로 egui 시각에 반영(캐시 키로 변경 시에만 적용).
- 설정창에 Theme 섹션: 테마 ComboBox(빌트인+커스텀), 스와치 미리보기, "Create your own theme" 에디터(색상 10종: 컬러픽커 + hex 텍스트), Save as theme / Delete this theme. 저장된 커스텀 테마는 `config.toml`의 `[custom_themes.<name>]`로도 직접 편집 가능.
- 빌드 경고 정리: `Stroke::new(1.0_f32, ...)` 명시.

### 2026-08-06 — 키 입력 라우팅 전면 재설계 + 클릭 안정화

이번 세션에서 해결한 핵심 문제들.

**1. ESC/숫자 키 무반응 (근본 원인 해결)**

- 증상: HUD는 뜨지만 ESC, 숫자 줌 키가 먹히지 않음. rdev 전역 후크가 키를 수신하지 못함(트리거 직후 숫자/ESC 로그 없음, 스퓨리어스 Alt 릴리즈만 도착).
- 결정: rdev 전역 캡처 → **egui 창 키 이벤트 직접 수신**으로 전환. `handle_hud_input`이 `egui::Key` → `MokeyKey` 매핑(`egui_key_to_mokey`) 후 처리.
- 결과: 숫자 줌, ESC 종료 모두 정상 동작. `capture` 플래그는 드래그 중 키 포워딩 전용으로 축소.

**2. 하위 셀 선택 중 자동 클릭**

- 증상: 4번째 줌부터 "클릭이 돼버림", 5번째 줌이 안 됨.
- 원인: `max_depth=4`에 도달하면 `auto_click=true`가 자동으로 클릭 + 세션 종료. 사용자가 더 깊게 줌하기를 원함.
- 수정: `auto_click` 기본값 `false`(클릭은 Enter/Space로 명시적), `max_depth` 기본값 4→6.

**3. 깊은 줌에서 렌더링 깨짐**

- 증상: depth 4~에서 셀이 라벨(26px 고정)보다 작아져 숫자가 겹쳐 보임.
- 수정: 셀 크기에 따라 라벨 폰트/테두리 두께 적응. 셀이 20px 미만이면 라벨 숨기고 테두리만 표시.

**4. Enter 클릭이 대상에 안 닿음 (스포티파이/브라우저 무반응)**

- 증상: 클릭은 발생(로그 `click=Some(Left)`)하지만 실제로는 아무것도 안 눌림.
- 원인: `ViewportCommand::OuterPosition`은 **프레임 끝에** 적용되는데, 기존 코드는 같은 프레임 안에서 30ms sleep 후 클릭을 주입 → mokey 창이 여전히 모니터를 덮고 있어 클릭이 mokey 창에 삼켜짐.
- 수정: 클릭/드래그 주입을 창이 실제로 화면 밖으로 나간 **다음 프레임(60ms 후)**으로 지연(`PendingAction`). 클릭 전 커서를 셀 중심으로 명시 이동.

**5. 진단 코드 정리**

- 트리거 후 20초 자동 종료(auto-exit) 진단 로직 제거.
- `Desktop\mokey-debug.log` 로깅을 `MOKEY_DEBUG=1` 환경변수로 게이트(기본 OFF). 디버깅 시 `$env:MOKEY_DEBUG=1` 후 실행하면 로그 활성.

### 2026-08-06 이전 — HUD 표시 안정화 (커밋 이력 기준)

- **검은 화면 HUD 수정**: 반투명 오버레이 대신 불투명 창 + 데스크톱 스크린샷 배경(`capture_monitor_bg`). egui 오버레이의 투명도 제약 회피.
- **시작 검은 사각형 제거**: 트리거 전까지 창을 화면 밖(`-32000,-32000`)에 유지.
- **유휴 시 HUD 얼어붙음 해결**: 채널로 전달된 핫키가 처리되도록 eframe 루프를 계속 깨움(wake callback + `request_repaint`).
- **전역 캡처 라우팅**: HUD 키를 전역 캡처로 라우팅하던 초기 설계(이후 egui 직접 수신으로 대체).

## 현재 상태

- Windows에서 트리거 → 줌(최대 6단계) → Enter 클릭 전체 흐름 동작 확인.
- 클릭 주입이 HUD 창에 삼켜지던 문제 해결 후 스포티파이/브라우저 대상 클릭 성공.
- 테스트: `cargo test` (mokey-core 20건 등 모두 통과).

## 남은 작업

- Phase 2: Hyprland(Wayland) 지원
- Vim 모드 드래그 릴리스 시점의 HUD 재표시 경합 등 엣지 케이스 검증
- 설정 UI의 슬라이더 범위(`max_depth` 최대 6) — 더 깊은 줌이 필요하면 확장
