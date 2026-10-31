# 용어집

| 용어 | 코드 이름 | 뜻 |
|---|---|---|
| 자모 | `Jamo` | 한글 낱자. 자음(`Consonant`)과 모음(`Vowel`)으로 나뉜다. 호환 자모 문자(U+3131~U+3163)로 표현한다 |
| 초성 / 중성 / 종성 | `cho` / `jung` / `jong` | 음절의 첫소리·가운뎃소리·끝소리 |
| 조합기 | `Composer` | 자모를 받아 음절 하나를 조합하는 상태 기계. 조합 중인 글자를 갖는다 |
| 조합 중 글자 | `preedit` | 아직 확정되지 않아 밑줄로 보이는 글자 |
| 확정 | `commit` | 조합 중 글자나 완성된 글자를 앱에 넘기는 일 |
| 자판 | `layout` | 키를 자모로 바꾸는 규칙. 지금은 두벌식 표준 하나 |
| 입력 모드 | `Mode` | `Hangul` 또는 `Latin`. 한/영 전환 키로 바뀐다 |
| 한/영 전환 키 | `Hotkey`, `toggle` | 입력 모드를 바꾸는 키. 키 이름과 수식 키(`Shift+space`)로 적고 설정 파일 `toggle =`로 바꾼다 |
| 수식 키 | `Modifiers` | Shift·Ctrl·Alt·Super. CapsLock·NumLock은 넣지 않는다 |
| 설정 | `config` | `~/.config/hangul-setting/config`. 지금은 `toggle` 하나 |
| 엔진 | `Engine` | 키 입력 하나를 받아 확정·조합 중 글자·키 소비 여부를 정하는 순수 로직 |
| 키 결과 | `KeyOutcome` | 엔진이 키 하나에 낸 결과 |
| 키심 | `keysym` | X11이 키에 붙인 기호 번호. 키코드를 자판 매핑으로 바꾼 값 |
| XIM | `xim` | X Input Method. 앱(클라이언트)과 입력기(서버)가 X 서버를 통해 주고받는 프로토콜 |
| 프론트엔드 | `xim_handler`, `ibus`, `gtk4-im` | 앱과 입력기 사이의 경로 하나를 맡는 부분. 모두 같은 엔진을 쓴다 |
| 조합 창 | `PreeditWindow` | 조합 중 글자를 스스로 그리지 않는 XIM 앱을 위해 커서 자리에 띄우는 작은 창 |
| GTK4 모듈 | `gtk4-im` | GTK4 앱이 읽는 입력 모듈(`GTK_IM_MODULE=hangul-setting`). 엔진을 앱 안에서 돌린다 |
| IBus 포털 | `ibus` | 세션 D-Bus의 `org.freedesktop.portal.IBus`. Qt 내장 ibus 플러그인이 `IBUS_USE_PORTAL=1`일 때 붙는 곳. 규격만 쓰고 ibus 소프트웨어는 쓰지 않는다 |
| 입력 컨텍스트 | `InputContext` | 앱 입력 창 하나에 대응하는 세션(XIM의 IC, IBus의 InputContext 객체). 조합 상태는 컨텍스트마다 따로 둔다 |
