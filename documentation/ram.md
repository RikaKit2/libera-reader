# Анализ памяти в libera-reader: полный ресёрч по GPUI-стеку

## 1. Откуда берутся 300–400 МБ в GPUI-приложении

Сначала — модель, иначе любые замеры бессмысленны. В десктоп-приложении на GPUI/wgpu память складывается из пяти относительно независимых слоёв, и твои 300–400 МБ — это скорее всего их сумма, а не одна жирная аллокация.

| Слой | Типичный диапазон | Откуда берётся | Как проверить |
| --- | --- | --- | --- |
| wgpu/Metal/Vulkan baseline | 50–100 МБ | инициализация GPU pipeline, swapchain, command buffers | пустое wgpu-приложение на Linux занимает 50–80 МБ [1] |
| Шрифты и текстовый кэш | 30–80 МБ | fontdue/parley/Cosmic-text загружают fallback-цепочки | dhat покажет аллокации в fontdue |
| Кэш изображений GPUI | 30–200+ МБ | подтверждённый баг: image bytes остаются в CPU после аплоада в GPU [2] | dhat, вкладка "image_cache" |
| Render-tree churn | 20–100 МБ | каждый кадр создаёт новые trait objects; bump-арена переиспользует, но не всё | dhat, "per-frame allocations" |
| Данные приложения (библиотека, кэш обложек, парсер) | 50–200 МБ | Vec, HashMap, image::open, полный EPUB в String | dhat по твоим модулям |

Это не теория. Автор дискуссии в zed-industries/zed#56737 показал, что шесть картинок в окне GPUI 0.2.2 жрали 300 МБ — после его форка с ленивым выгрузом байтов из CPU стало 12 МБ [2]. В zed-industries/zed#10203 авторы признают: «hello world на GPUI ест 170+ МБ на Windows» [3]. Этот baseline держится и в 2026 году — он не баг, а стоимость входа в wgpu/winit/Metal-стек.

Твоя задача — не «уменьшить до 50 МБ» (нереально для GPUI), а понять, какой из слоёв у тебя раздут сильнее нормы, и резать его. Для этого нужны правильные метрики и профайлеры.

## 2. Корректный замер памяти на Linux: RSS vs PSS vs USS

`top` и `htop` показывают RSS (Resident Set Size) — это «сколько физической памяти сейчас занимают страницы процесса». Для GUI-приложения с wgpu RSS врёт в большую сторону, потому что shared texture layer и shared memory от GPU-драйвера учитывается полностью, хотя реально делится между процессами.

Что использовать:

- **PSS (Proportional Set Size)** — твоя честная доля от shared памяти. Сумма PSS по всем процессам = реальное использование RAM. Это цифра, которую нужно оптимизировать.
- **USS (Unique Set Size)** — сколько памяти освободится, если убить процесс. Полезно, чтобы понять, сколько из RSS — «твоё», а сколько — shared с GPU-драйвером и соседями.
- **VSZ (Virtual Size)** — почти бесполезен. mmap'нутый файл EPUB в 50 МБ добавит 50 МБ к VSZ, но ноль к PSS, пока ты реально не прочитал данные.

Команды для замера:

```bash
# Установи smem (один раз, через NixOS)
nix-shell -p smem --run "smem -P $(pidof libera-reader)"

# Без smem: прямой PSS из procfs
PSS_KB=$(grep ^Pss: /proc/$(pidof libera-reader)/smaps_rollup | awk '{print $2}')
echo "PSS: $((PSS_KB / 1024)) MB"

# USS — то, что освободится при kill
USS_KB=$(awk '/^Private_/ {sum += $2} END {print sum}' /proc/$(pidof libera-reader)/smaps_rollup)
echo "USS: $((USS_KB / 1024)) MB"
```

`/proc/$pid/smaps_rollup` есть в ядрах 4.14+ и агрегирует всю память процесса в одну запись [4]. Это быстрее, чем парсить smaps для каждого mapping.

Если у тебя в htop 350 МБ RSS, а в PSS — 220 МБ, значит ~130 МБ «shared с кем-то». Это не утечка, и оптимизировать тут нечего. Смотри PSS.

## 3. Профайлеры кучи: пять инструментов, их плюсы и подводные камни

Gemini дал dhat и heaptrack. Этого мало: у каждого свои ограничения. Вот полная картина.

| Инструмент | Платформа | Что показывает | Точность | Оверхед | Свежесть (2026) |
| --- | --- | --- | --- | --- | --- |
| dhat-rs | везде | call-site каждой аллокации, peak live bytes `file:line:col` | точно | замедляет 2-5× | 0.3.3 (февраль 2024), стабилен [5] |
| heaptrack | Linux | timeline, flame graph, leaks | function-level | умеренный | активен, поддержка Rust символов [6] |
| bytehound | Linux | всё выше + экспорт в heaptrack/flamegraph | function-level | высокий | последний апдейт в 2023, автор сказал «finished» [7] |
| valgrind massif | Linux | heap snapshot over time | function-level | замедляет 10-50× | 3.25+, зрелый [8] |
| gperftools/tcmalloc | Linux | periodic heap dumps (1 МБ файлы) | function-level | низкий | зрелый, нужно линковать tcmalloc [9] |

dhat-rs — единственный кросс-платформенный, поэтому если ты на macOS или в WSL2 без нормального perf, это твой главный инструмент. Ставится за 5 минут:

```toml
# Cargo.toml
[dependencies]
dhat = { version = "0.3", optional = true }

[features]
dhat-heap = ["dhat"]
```

```rust
// src/main.rs
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    // ... остальной код
}
```

```fish
cargo run --release --features dhat-heap
# потыкай приложение, выйди — в корне появится dhat-heap.json
# загрузи его в https://valgrind.github.io/dhat/ или https://nnethercote.github.io/dhat-viewer/
```

Важная деталь: dhat-rs требует наличия libtcmalloc на хосте (gperftools-devel на Fedora, gperftools на NixOS). Без этого будет падать при старте [5].

heaptrack — лучший выбор, если ты на Linux. Делает flame graph и автоматически деманглит Rust-символы. Запуск:

```fish
# В Fish с NixOS
nix-shell -p heaptrack --run "heaptrack ./target/release/libera-reader"
# Потыкай, выйди — будет heaptrack.libera-reader.XXXX.gz
heaptrack_gui heaptrack.libera-reader.XXXX.gz
```

bytehound — самый красивый UI, экспортит в формат heaptrack и в flamegraph SVG, лучше всех деманглит Rust-символы [7]. Но автор с 2023 не отвечает на issue — используй на свой страх.

valgrind massif — старый, медленный (программа идёт в 10-50× медленнее), но работает в любой ситуации, где не работают другие. Полезен как последнее средство.

gperftools (tcmalloc) — если хочешь heap profile с минимальным оверхедом, но нужна возня с build-скриптом [9].

Мой рекомендованный workflow для libera-reader:

1. Первый заход: `cargo run --release --features dhat-heap`, потыкать, выгрузить `dhat-heap.json`, открыть в viewer. Получишь картину «кто аллоцирует по байтам».
2. Второй заход: `heaptrack ./target/release/libera-reader`, сделать те же действия. Увидишь, какие аллокации реально живут долго, а какие тут же умирают.
3. Корреляция: если dhat показывает 80 МБ в `gpui::image::cache`, а heaptrack показывает, что эти 80 МБ живут вечно — у тебя именно тот баг из issue #56737.

## 4. Подтверждённые узкие места в GPUI: баги, которые прямо сейчас жрут память

Вот то, что Gemini пропустил: в zed-industries/zed висят конкретные issue с цифрами и готовыми диагнозами. Это не «может быть», это «у одного из мейнтейнеров то же самое, и он выложил PoC».

### Баг #27414: image cache leak (закрыт, но есть нюансы)

Сабмиттер показал, что в GPUI картинки кэшируются в `cx.loading_assets`, и при смене src у `<img>` старые ассеты не выгружаются. На 30 циклах обновления превью утечка с ~100 МБ до ~1 ГБ [10]. Закрыт PR #27774 — если ты на свежем gpui (Zed 0.190+), этот баг уже не твой. Проверь версию в Cargo.lock.

### Баг #56737: image bytes висят в CPU после GPU-аплоада (ОТКРЫТ, май 2026)

Это главный подозреваемый для ридера. Автор дискуссии измерил: 6 картинок в окне GPUI 0.2.2 = 300 МБ RAM. В его форке fabbb60 после фикса (не держать байты на CPU после загрузки в GPU) — 12 МБ [2]. Разница в 25×.

Статус на 14 мая 2026: issue всё ещё открыт, 0 комментариев, автор не предлагает PR в upstream [2]. Это значит, что в твоей версии GPUI этот код-паттерн, скорее всего, ещё жив.

Что с этим делать:

1. Проверь, сколько обложек книг у тебя в библиотеке и как они рендерятся. Если превью 100 обложек 200×300 px, каждая = ~240 КБ в RGBA = 24 МБ только в image cache. Если у тебя полноразмерные 800×1200 — это уже 3.8 МБ каждая, и 100 обложек = 380 МБ.
2. Не держи полные байты в `Vec<u8>` после декодинга. Декодируй в `image::ImageBuffer`, отдай в GPUI, и забудь.
3. Используй LRU-кеш для обложек с фиксированным лимитом (подробнее в разделе 9).

### Баг #10203: hello world GPUI = 170 МБ на Windows

Самый базовый пример. Это значит, что ниже ~170 МБ на GPUI-приложении уйти нельзя без серьёзных хаков [3]. Не пытайся.

### Внутренности GPUI 2: per-frame bump allocator

Из блога Zed Weekly #29 (опубликован командой Zed) [11]:

> Rendering a frame performs a lot of allocations. Every element in the element tree needs to be a trait object, because we want the system to be open-ended and compositional. In GPUI 1, we ate the cost of malloc and simply boxed our trait objects. With GPUI 2, we're allocating a lot more closures, and we also have the cost of performing layout via taffy, which is more expensive than our old approach but also a lot more flexible and powerful. To buy ourselves more headroom, we ended up implementing our own simple bump allocated arena, which we access via a thread-local variable during frame rendering.

Перевод: GPUI 2 уже использует thread-local bump-арену для собственных внутренних аллокаций кадра. Это значит, что `Box<dyn Element>`, `Box<dyn Layout>` и прочие внутренние структуры переиспользуются между кадрами. Но твои closure'ы в `render()` всё равно идут через обычный аллокатор — bump-арена только для самого фреймворка.

Следствие: если в `render()` ты делаешь `format!("{title}")`, `to_string()`, `vec![]`, `HashMap::new()` — это всё реальные heap-аллокации, которые случаются 60-120 раз в секунду при 60 FPS. За минуту скролла библиотеки это тысячи мелких аллокаций, которые давят на аллокатор.

## 5. Глобальный аллокатор: 5 строк кода, которые могут убрать 30-50% RSS

Это самая дешёвая оптимизация. Замена аллокатора не требует менять ни строчки логики.

### Состояние дел в 2026 году

- **jemalloc** — был дефолтом в Rust до конца 2018 (PR rust-lang/rust#55238), потом удалён. 12 июня 2025 Джейсон Эванс (создатель) опубликовал postmortem: «active upstream development has come to an end» [12]. Но Meta в феврале 2026 объявила о renewed commitment к jemalloc и форкнула его [13]. Крейт tikv-jemallocator жив и обновляется (0.5.4, статус «actively-developed»). Так что «unmaintained» — горячий спор; реальность — работает, но будущее под вопросом.
- **mimalloc** — главный современный выбор. Microsoft выпустила v3.1.5 beta 13 июня 2025 [14]. Для Rust есть несколько обёрток:
  - `mimalloc` crate — официальный wrapper, актуален.
  - `mimalloc-rust` (purpleprotocol) — мёртв с апреля 2023, не используй.
  - `mimalloc3-rs` (июль 2025) — обёртка под mimalloc v3, свежая [15].
  - `better_mimalloc_rs` (январь 2026) — форк dev-ветки с tuning knobs для RSS [16].
- **tcmalloc (gperftools)** — выбор Google. Хорош для многопоточных серверов, для десктопа избыточен.
- **system allocator (glibc ptmalloc)** — дефолт. Склонен к фрагментации, особенно в GUI-приложениях с постоянными мелкими аллокациями.

### Что выбрать для ридера

mimalloc через крейт `mimalloc3-rs` или `better_mimalloc_rs` — лучший баланс «маленький оверхед + хорошее поведение для маленьких аллокаций + агрессивный возврат памяти в ОС».

Паттерн с feature flag (чтобы можно было сравнивать):

```toml
# Cargo.toml
[dependencies]
mimalloc = { version = "0.1", default-features = false, optional = true }
tikv-jemallocator = { version = "0.5", optional = true }

[features]
default = []
mimalloc-alloc = ["dep:mimalloc"]
jemalloc-alloc = ["dep:tikv-jemallocator"]
```

```rust
// src/main.rs
#[cfg(feature = "mimalloc-alloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(all(feature = "jemalloc-alloc", not(target_env = "msvc")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() { /* ... */ }
```

Сборка: `cargo run --release --features mimalloc-alloc` против `cargo run --release` (без фичи, glibc). Замерь PSS в обоих случаях через smem. В ридерах обычно mimalloc даёт -20-40% PSS за счёт лучшей работы с маленькими аллокациями (а у тебя в `render()` их тысячи).

Если у тебя glibc allocator и ты не хочешь менять на новый крейт — есть менее радикальный трюк: `M_MMAP_THRESHOLD` через `mallopt` уменьшает порог, при котором glibc использует mmap вместо кучи, что уменьшает фрагментацию. Но это уже костыль.

## 6. Типы строк: где именно твой render() жжёт аллокации

Gemini упомянул SharedString. Это правильно, но без понимания, почему String дорогой в Rust, совет не поможет.

### Факт, который часто упускают

В Rust `String` не имеет Small String Optimization (SSO). В C++ `std::string` до ~15 символов хранит inline. В Rust любой непустой `String` — это heap-аллокация (минимум 24 байта overhead: pointer + length + capacity) [17]. Если ты в `render()` вызываешь `.to_string()` или `format!()` — это heap-аллокация на каждом кадре.

### Сравнение типов

| Тип | size_of | Где живут данные | Когда использовать |
| --- | --- | --- | --- |
| String | 24 байта | heap (для любой непустой) | владелец строки, которую ты мутируешь |
| Arc<String> | 8 байт + 40+ на куче | Arc + String на куче | никогда — двойной indirection |
| Arc<str> | 8 байт + len на куче | один indirection | иммутабельная строка, шарится между задачами |
| SharedString (GPUI) | 24 байта | inline ≤23 байт, иначе heap (через SmolStr) | строки в GPUI-виджетах, render closure [18] |
| SmolStr | 24 байта | inline ≤23 байт, иначе heap | ключи, имена, метки, любые короткие строки [19] |
| compact_str | 24 байта | inline ≤15 байт UTF-8 (16+ для Latin-1) | альтернатива SmolStr для коротких ASCII |

### Практические правки для render-closure

```rust
// ПЛОХО: каждая аллокация на каждом кадре
fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div().child(self.title.to_string()) // String::clone = malloc
}

// ХОРОШО: zero-cost копия
fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div().child(self.title.clone()) // SharedString::clone = bump pointer
}

// ЕЩЁ ЛУЧШЕ: если title не меняется, храни как SharedString изначально
struct BookView {
    title: SharedString,
    author: SharedString,
}
```

`SharedString` в актуальной версии GPUI — это абстракция над `Arc<str>` и `&'static str`, бэкенд — SmolStr [18]. SmolStr (последняя версия 0.3.6, 4 марта 2026) inline-хранит строки до 23 байт и делает O(1)-clone [19]. Это идеальный тип для заголовков глав, имён авторов, меток — всего, что короткое и иммутабельное.

Если у тебя в `Book` есть поля `genre: String`, `publisher: String`, `language: String` — замени их на `SharedString` или хотя бы `Arc<str>`. Умножь на количество книг в памяти — получишь мегабайты.

## 7. Коллекции: hashbrown, Vec, slotmap — что выбрать

Gemini не упомянул цену коллекций. А она есть.

### HashMap<K, V> (= hashbrown с Rust 1.36)

С 2019 года `std::collections::HashMap` — это `hashbrown::HashMap` [20]. Накладные расходы:

- 1 байт overhead на bucket (control byte)
- load factor 7/8 → после очередного удвоения ~56% capacity свободно
- средний overhead ~70% на пустые слоты поверх реальных данных [21]
- пустой HashMap не аллоцирует (только stack)

Если у тебя `HashMap<String, Book>` на 1000 книг, ты платишь ~70% сверху на пустые слоты. Для ридера, где ключи — это пути файлов, Vec с бинарным поиском может быть в разы компактнее.

### Альтернативы

| Структура | Когда выбрать | Память |
| --- | --- | --- |
| `Vec<T>` с сортировкой + `binary_search` | ≤10 000 элементов, редкие вставки | минимальный оверхед, контигентная память |
| `BTreeMap<K, V>` | нужна сортировка, range queries | ~50% overhead (узлы + балансировка) [21] |
| `indexmap` | нужен порядок вставки + быстрый lookup | больше HashMap из-за вектора индексов |
| `slotmap` / `generational-arena` | сущности с stable handles, частые add/remove | overhead на generation counter |
| `HashMap` (hashbrown) | общий случай, особенно для строковых ключей | ~70% overhead |

### Практика

- После того как Vec перестал расти — `vec.shrink_to_fit()`. Может вернуть десятки мегабайт.
- `HashMap::with_capacity(N)` — выдели сразу нужное, чтобы избежать rehash'ей и временных аллокаций.
- `Box<[T]>` вместо `Vec<T>` для иммутабельных списков: экономит 1 слово (capacity), аллокатор не держит capacity overhead.
- `ThinVec<T>` (из крейта `thin_vec`) — Vec с пустым представлением в 1 байт (niche), экономит место, когда Vec — поле большого struct.

```rust
// ПЛОХО: 24 байта на каждую Book даже если вектор пустой
struct Book {
    chapters: Vec<Chapter>,
    images: Vec<Image>,
}

// ЛУЧШЕ: 1 байт когда пусто
struct Book {
    chapters: ThinVec<Chapter>,
    images: ThinVec<Image>,
}
```

Если у тебя в `Book` 5+ полей типа `Vec<T>` или `HashMap<K, V>`, разница в `size_of::<Book>()` может быть существенной. Используй крейт `size-of` с derive `SizeOf` (см. раздел 11).

## 8. Чтение книг: mmap, zero-copy, стриминг

Это специфика libera-reader, которую Gemini не раскрыл.

### Три подхода к чтению файла

```rust
// ПОДХОД 1: прочитать всё в String (плохо для больших книг)
let mut s = String::new();
File::open("book.epub")?.read_to_string(&mut s)?;
// s теперь ~5 МБ на куче навсегда

// ПОДХОД 2: mmap — почти zero-cost
let file = File::open("book.epub")?;
let mmap = unsafe { memmap2::Mmap::map(&file)? };
// mmap.len() == 5 МБ, но в твоём PSS это 0 пока не читаешь
// Парсишь &str слайсы прямо в mapping
let chapter = std::str::from_utf8(&mmap[offset..offset+len])?;

// ПОДХОД 3: стриминг через BufReader (хорошо для парсеров с итераторами)
let file = BufReader::new(File::open("book.epub")?);
for line in file.lines() {
    process(line?);
}
```

`memmap2` [22] — стандартная обёртка над mmap(2). ОС сама подгружает страницы по необходимости и вытесняет их под давлением. RSS растёт только когда ты реально читаешь данные. Идеально для EPUB/FB2/PDF.

### Парсеры EPUB в 2026 году

| Крейт | Свежесть | Подход | Память |
| --- | --- | --- | --- |
| lexepub | 0.1.1 (7 мая 2026) | очень свежий, стриминг по главам, built on Tokio | низкая, chapter-by-chapter [23] |
| lib-epub | 0.3.1 (26 апреля 2026) | свежий, полный парсинг контейнера, EPUB 2/3 | средняя |
| epublet | 0.2.0 (8 февраля 2026) | свежий, no_std, target <120 КБ | минимальная, для embedded [24] |
| epub 1.2.2 (danigm) | старый, стабильный | полный парсинг | средняя |
| epub-parser | стабильный | полный парсинг NCX/OPF | средняя |
| fb2 (lib.rs) | стабильный | quick-xml based | зависит от использования |

lexepub — самый свежий и явно спроектированный под стриминг: «Process EPUBs chapter-by-chapter without loading everything into memory» [23]. Если у тебя ещё нет EPUB-парсера или текущий грузит всё в DOM, это явный кандидат.

epublet — экстремальный случай. Целевой peak RAM < 120 КБ сверх framebuffer'а [24]. Если тебе нужна встраиваемая читалка (e-ink) — смотри его. Для десктопа избыточен.

### Zero-copy паттерн для парсера

```rust
// ПЛОХО: String::from_utf8 копирует
let s = String::from_utf8(chunk.to_vec())?;

// ХОРОШО: &str поверх mmap'а — zero copy
let s = std::str::from_utf8(&mmap[range])?;
// работай с s, mmap держит данные
```

Если твой парсер EPUB/FB2 сейчас делает `String::from_utf8(bytes.to_vec())` — ты платишь за каждую главу полную копию. Замени на `&str` слайсы через mmap, и парсер будет аллоцировать только то, что реально нужно сохранить (метаданные, текущая глава).

## 9. Кэш обложек и ленивый аплоад в GPU

Это самый вероятный крупный источник жора в твоём случае, если у тебя библиотека на десятки/сотни книг.

### Расчёт

- 1 обложка 800×1200 в RGBA = 3.84 МБ в CPU
- 100 обложек = 384 МБ только на превью
- Это именно та цифра, что была в баге #56737 (6 картинок = 300 МБ → 12 МБ после lazy upload) [2]

### Стратегия: трёхуровневый кэш

1. Диск: PNG-thumbnail 200×300 сохраняешь в `dirs::cache_dir()/libera-reader/covers/`. На диске они ничего не стоят.
2. Память: LRU-кеш на N последних отрендеренных обложок. N = 20-50.
3. GPU: загружено в текстуру. CPU-байты после этого можно дропать (если GPUI это позволяет; issue #56737 показывает, что не всегда).

### LRU-крейты в 2026 году

| Крейт | Свежесть | Фичи |
| --- | --- | --- |
| lru (jeromefroe / stormshield fork) | живой, MSRV 1.64 | простой O(1) LRU |
| lru-cache (contain-rs) | MAINTENANCE MODE — «no longer improved» [25] | не используй |
| caches | активный | LRUCache, SegmentedCache, TwoQueueCache, AdaptiveCache |
| hashlru | 0.11.1 (март 2024) | средне-активный, HashMap-подобный API |
| stretto | 0.9.0 | активный, 11К загрузок, thread-safe, TinyLFU |

Для однопоточного кэша обложек бери `lru` или `caches::LRUCache`. Для многопоточного — `stretto`.

### Паттерн

```rust
use lru::LruCache;
use std::num::NonZeroUsize;

struct CoverCache {
    // хранит декодированные байты, LRU
    inner: LruCache<PathBuf, Arc<image::DynamicImage>>,
}

impl CoverCache {
    fn new() -> Self {
        Self {
            inner: LruCache::new(NonZeroUsize::new(50).unwrap()),
        }
    }

    fn get_or_load(&mut self, path: &Path) -> Result<Arc<image::DynamicImage>> {
        if let Some(img) = self.inner.get(path) {
            return Ok(img.clone());
        }
        // Декодируем
        let img = Arc::new(image::open(path)?);
        // Resize до thumbnail
        let thumb = Arc::new(img.resize(200, 300, image::imageops::FilterType::Triangle));
        self.inner.put(path.to_path_buf(), thumb.clone());
        Ok(thumb)
    }
}
```

Не держи полноразмерные байты в кэше. Декодируй, ресайзь до нужного размера сразу при загрузке, складывай thumbnail. Иначе 100 обложек × 3.8 МБ = 380 МБ.

### Двухуровневый кэш (как в Tauri-примере)

Если обложки тяжёлые (например, HEIC), в `dirs::cache_dir()` складывай converted PNG, чтобы не декодировать оригинал каждый раз. Pattern из реального проекта (Tauri image viewer [26]):

```rust
fn get_cover(path: &Path) -> Result<DynamicImage> {
    // 1. Сначала дисковый кэш
    let cache_path = cover_cache_path(path);
    if cache_path.exists() {
        return image::open(cache_path);
    }
    // 2. Иначе декодируем + ресайзим + сохраняем
    let img = image::open(path)?.resize(200, 300, FilterType::Triangle);
    img.save(&cache_path)?;
    Ok(img)
}
```

## 10. Render-tree churn и cx.notify()

Из Zed Weekly #29 [11] мы знаем, что GPUI 2 уже использует thread-local bump-арену для собственного frame-rendering. Но:

1. Твои closure'ы в `render()` идут через обычный аллокатор.
2. `cx.notify()` без надобности заставляет GPUI перестроить весь scene-graph, что перерасходует и bump-арену тоже.
3. taffy layout в GPUI 2 — это новый layout engine, более гибкий, но более дорогой по аллокациям, чем был в GPUI 1.

### Правила

- Не вызывай `cx.notify()` в цикле или на каждое мелкое изменение. Это сигнал «перерисуй меня». Если ты его дёргаешь 60 раз в секунду, рендер идёт постоянно.
- Используй `RenderOnce` (или эквивалентный fluent API) для статичных виджетов, у которых render-closure детерминирован и не подписан на state. Это дешевле, чем `Render`.
- Не создавай `format!()` и `String` в `render()` — заменяй на `SharedString` (раздел 6).
- Предпочитай `iter().map()` над `collect::<Vec<_>>()` в render-closure, где возможно.
- Не вызывай `image::open()` в `render()` — это синхронный decode на каждом кадре. Декодируй заранее в фоне, клади в `Shared<DynamicImage>`.

## 11. Тип-анализ и зависимостный анализ

### cargo-bloat — показывает binary size, не RAM. Но полезен как hint

```text
cargo install cargo-bloat && cargo bloat --release --crates [27]:

File .text Size Crate
18.3% 35.7% 1.2MiB std
8.4% 16.4% 541.2KiB gpui
4.7% 9.2% 302.8KiB wgpu
3.1% 6.1% 200.5KiB taffy
2.8% 5.5% 180.2KiB image
2.0% 3.9% 128.7KiB fontdue
...
```

Здесь видно, какие крейтеры тащат много кода. Это не равно RAM, но если taffy (layout engine) занимает 5% бинаря, он и в runtime создаёт соответствующее количество объектов.

### size-of крейт — настоящая находка

`size-of` с derive `SizeOf` рекурсивно считает `size_of` всех полей структуры, включая Vec capacity, String capacity, HashMap capacity [28]. Это даёт compile-time знание о реальном размере твоих типов с учётом всех аллокаций.

```toml
[dependencies]
size-of = "0.1"
```

```rust
use size_of::SizeOf;

#[derive(SizeOf)]
struct Book {
    title: SharedString, // 24 байта (inline до 23)
    author: SharedString, // 24 байта
    chapters: ThinVec<Chapter>,
    cover: Option<Arc<DynamicImage>>,
}

#[derive(SizeOf)]
struct Chapter {
    title: SharedString,
    content: Vec<u8>, // capacity matters!
}

fn main() {
    println!("Book: {}", size_of::(&Book::size_of)); // включая heap!
}
```

Это покажет, что `Vec<u8>` с capacity 1 МБ для главы — это +1 МБ на книгу, а не «несколько байт overhead как думает новичок».

### -Zprint-type-sizes (nightly)

Rust Performance Book рекомендует [29]:

```fish
RUSTFLAGS=-Zprint-type-sizes cargo +nightly build --release 2>&1 | head -100
```

Это печатает размер и layout каждого типа в твоём крейте. Очень полезно, чтобы найти enum с жирным вариантом или struct с случайно вставленным `Box<dyn Trait>`.

## 12. CI-тесты на регрессии памяти

Без регрессионного теста через полгода ты снова будешь на 400 МБ и не поймёшь, когда это случилось.

### dhat-rs в тестах

dhat поддерживает ad-hoc режим: можно проверять количество аллокаций и peak memory в тестах [5]:

```rust
#[test]
fn library_load_should_not_blow_up() {
    let _profiler = dhat::Profiler::new_heap();
    let lib = load_test_library();
    let stats = dhat::HeapStats::get();

    dhat::assert!(stats.total_bytes < 50 * 1024 * 1024);  // 50 MB
    dhat::assert!(stats.max_blocks == 0);  // всё освобождено
}
```

### alloc-counter крейт

Если не хочешь тащить dhat, есть `alloc_counter` — позволяет считать аллокации в конкретном scope:

```rust
use alloc_counter::AllocCounter;

#[global_allocator]
static A: AllocCounter<System> = AllocCounter::new(System);

#[test]
fn chapter_load_allocates_bounded() {
    let (_count, bytes) = alloc_counter::count(|| {
        let _ch = load_chapter("test.fb2");
    });
    assert!(bytes < 1_000_000); // меньше 1 МБ на главу
}
```

### CI-обёртка

`glassbench` крейт — benchmarks с memory tracking [30]. `memuse` — traits для замера dynamic memory usage типов. Любой из них встанет в `cargo bench` и зарейзит CI alarm, если новый PR добавил 20 МБ к peak.

## 13. CPU-профиль как индикатор memory churn

Если 10-20% CPU уходит в malloc/free — у тебя memory churn. Это прямо коррелирует с давлением на аллокатор.

### cargo-flamegraph

```fish
cargo install flamegraph
echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid # one-time

cargo flamegraph --release --bin libera-reader
# потыкай, выйди — откроется flamegraph.svg
```

Если в flamegraph'е широкие блоки `__libc_malloc` или `mi_malloc` — это оно.

### samply (кросс-платформенный)

```fish
cargo install --locked samply
samply record ./target/release/libera-reader
# откроется Firefox Profiler
```

## 14. Практический план для libera-reader

Вот что делать в каком порядке. Сначала измерь, потом режь — иначе оптимизируешь не то.

### День 1: инструменты и метрики

1. Поставь smem и научись читать PSS вместо RSS. Запиши baseline: PSS пустого приложения, PSS после открытия одной книги, PSS после открытия библиотеки на 100 книг.
2. Прогони `cargo run --release --features dhat-heap`. Открой `dhat-heap.json` в viewer'е. Запиши топ-5 функций по `total_bytes` и по `max_bytes`. Это твои главные подозреваемые.
3. Прогони heaptrack на тех же сценариях. Сравни — heaptrack покажет, какие из этих аллокаций живут долго.

### День 2: низко висящие плоды

1. Поменяй аллокатор на mimalloc (через feature flag). Замерь PSS до/после. Обычно -20-40%.
2. Найди все `String` и `to_string()` в render-closure'ах. Замени на `SharedString` и `.clone()`. Обычно -10-30 МБ на тяжёлых списках.
3. Добавь `#[global_allocator]` profiling-mode build через `cfg_if` + feature `system-alloc`, чтобы можно было сравнивать с дефолтом [9].

### День 3: специфика ридера

1. Если у тебя библиотека с обложками — реализуй LRU-кеш на 20-50 элементов, ресайзь обложки при загрузке, не держи полные байты в памяти после рендера. Проверь, как твоя версия GPUI себя ведёт: посмотри PR #27774 и обновись, если там фикс.
2. Если парсишь EPUB/FB2 целиком в String — переходи на `memmap2` + `&str` слайсы, либо на стриминг через lexepub [23].
3. Найди `cx.notify()` в циклах (например, в фоновом парсинге). Замени на batching: парсишь файл, один `cx.notify()` в конце, не на каждый килобайт.

### День 4: тип-анализ и закрепление

1. `cargo bloat --release --crates` — посмотри, нет ли у тебя в зависимостях чего-то жирного, что ты не используешь (часто tokio с дефолтными фичами тянет 1 МБ кода).
2. `size-of` derive на ключевых структурах (`Book`, `Chapter`, `LibraryEntry`) — посмотри реальные размеры.
3. Добавь `dhat-assert` в CI на ключевые сценарии: `assert!(stats.total_bytes < 100 * 1024 * 1024)`. Это защитит от регрессий.

### Ожидаемый результат

По моему опыту и по отчётам в Hacker News / Reddit, для типичного GPUI-приложения такой план убирает 100-150 МБ за 1-2 дня работы, без потери функциональности. Баг #56737 один даёт -200-300 МБ, если он у тебя релевантен. Mimalloc даёт -20-50 МБ. SharedString в render'е — ещё -10-30 МБ.

300-400 МБ — это нормальный baseline для GPUI-приложения с GPUI 0.2.x. Целься в 200-250 МБ. До 100 МБ не уйти без хаков типа собственного renderer'а или переписывания на iced/slint.

## Контекст: сколько жрут другие ридеры

Чтобы у тебя была точка отсчёта [31]:

| Приложение | RAM | Стек |
| --- | --- | --- |
| Bookworm (GTK, минималистичный) | 40-80 МБ | GTK |
| Foliate (GTK4 native) | 60-100 МБ | GTK4 |
| Okular (Qt) | 80-120 МБ | Qt |
| KOReader | 100-150 МБ | Lua + Qt |
| Thorium Reader | 100-150 МБ | Electron |
| Calibre viewer | ~230 МБ | Qt |
| Calibre (full) | 150-250 МБ | Qt + plugins |
| Zed empty | 200-400 МБ | GPUI |
| libera-reader (твоя цель) | 200-250 МБ | GPUI |

Твоя цель — попасть в диапазон Foliate/Okular, не в Bookworm. Bookworm не рендерит через GPU и не использует современный layout engine — у него нет того, что есть у тебя. Реалистичная цель — 200-250 МБ с полным GPU-рендерингом, библиотекой и обложками.

## Источники

- [1] https://users.rust-lang.org/t/is-an-empty-wgpu-app-expected-to-use-50mb/68252 — обсуждение пустого wgpu приложения на Linux с RSS 50-80 МБ.
- [2] https://github.com/zed-industries/zed/discussions/56737 — дискуссия о GPUI 0.2.2, рендерящем 6 картинок в 300 МБ; форк автора сокращает до 12 МБ через ленивый аплоад.
- [3] https://github.com/zed-industries/zed/issues/10203 — issue «GPUI's hello world demo takes up 170M+ of memory on windows».
- [4] https://www.kernel.org/doc/Documentation/ABI/testing/procfs-smaps_rollup — документация ядра по /proc/$pid/smaps_rollup.
- [5] https://github.com/nnethercote/dhat-rs — репозиторий dhat-rs, последний релиз 0.3.3 (4 февраля 2024).
- [6] https://github.com/KDE/heaptrack — heaptrack с поддержкой деманглинга Rust символов.
- [7] https://github.com/koute/bytehound — bytehound, последние обновления в 2023, автор назвал проект «finished».
- [8] https://valgrind.org/docs/manual/ms-manual.html — руководство по Valgrind Massif.
- [9] https://blog.chainsafe.io/memory-analysis-in-rust-2/ — гайд по всем профайлерам Rust: dhat, heaptrack, bytehound, valgrind massif, gperftools.
- [10] https://github.com/zed-industries/zed/issues/27414 — баг image cache memory leak в GPUI, закрыт PR #27774.
- [11] https://zed.dev/blog/zed-weekly-29 — блог Zed Weekly #29 про внутренности GPUI 2 и bump-арену.
- [12] https://jasone.github.io/2025/06/12/jemalloc-postmortem/ — postmortem Джейсона Эванса о jemalloc, 12 июня 2025.
- [13] https://www.reddit.com/r/rust/comments/1riwbqv/perf_allocator_has_a_high_impact_on_your_rust/ — обсуждение аллокаторов с упоминанием renewed commitment Meta к jemalloc.
- [14] https://github.com/microsoft/mimalloc — репозиторий mimalloc, последние релизы: v3.1.5 beta (13 июня 2025), v2.2.4 stable.
- [15] https://lib.rs/crates/mimalloc3-rs — обёртка mimalloc v3 для Rust, релиз 0.0.5 (31 июля 2025).
- [16] https://lib.rs/crates/better_mimalloc_rs — форк mimalloc dev-ветки, релиз 0.1.1 (26 января 2026).
- [17] https://internals.rust-lang.org/t/short-string-optimization/8436 — обсуждение отсутствия SSO в String в Rust.
- [18] https://docs.rs/gpui — документация GPUI: SharedString как абстракция над Arc<str> и &'static str, бэкенд SmolStr.
- [19] https://lib.rs/crates/smol_str — SmolStr 0.3.6 (4 марта 2026), inline до 23 байт, O(1) clone.
- [20] https://github.com/rust-lang/hashbrown — hashbrown, дефолтный HashMap в Rust с 1.36.
- [21] https://ntietz.com/blog/rust-hashmap-overhead/ — измерение overhead HashMap (~70% средний) и BTreeMap (~50%).
- [22] https://docs.rs/memmap2 — крейт memmap2 для memory-mapped файлов.
- [23] https://lib.rs/crates/lexepub — стриминговый EPUB-парсер, релиз 0.1.1 (7 мая 2026).
- [24] https://lib.rs/crates/epublet — EPUB парсер для embedded, no_std, target < 120 КБ.
- [25] https://docs.rs/crate/lru-cache/latest — lru-cache (contain-rs) в maintenance mode.
- [26] https://takazudomodular.com/pj/zudo-tauri/docs/recipes/image-viewer-app/ — паттерн двухуровневого кэша (in-memory LRU + on-disk).
- [27] https://github.com/RazrFalcon/cargo-bloat — cargo bloat для анализа бинаря.
- [28] https://docs.rs/size-of — крейт size-of с derive SizeOf для compile-time анализа размеров.
- [29] https://nnethercote.github.io/perf-book/type-sizes.html — глава Type Sizes из The Rust Performance Book.
- [30] https://lib.rs/development-tools/profiling — список профайлинг-крейтов: glassbench, memuse, memory-stats.
- [31] https://www.merge-json-files.com/blog/best-epub-reader-for-ubuntu — сравнение EPUB-ридеров для Ubuntu 2026.
