//! ブラウザ向けのフロントエンド。`trunk` がこれを wasm32 向けにビルドする。
//!
//! シミュレーション自体は `human_emulator` ライブラリがそのまま担う。
//! ここにあるのは入力の受け取りと描画だけ。

#[cfg(target_arch = "wasm32")]
mod app {
    use human_emulator::{Activity, Course, Drive, Human};
    use web_sys::HtmlInputElement;
    use yew::prelude::*;

    const DAY_CHOICES: [u32; 4] = [3, 7, 14, 30];

    #[component(App)]
    pub fn app() -> Html {
        let seed = use_state(|| 42u64);
        let days = use_state(|| 7u32);
        let name = use_state(|| "被験者".to_string());
        let selected = use_state(|| 0u32);

        let human = {
            let mut h = Human::new((*name).clone(), *seed);
            h.run(*days);
            h
        };
        // 日数を減らしたときに選択が範囲外へ出ないようにする。
        let selected_day = (*selected).min(days.saturating_sub(1));

        let on_seed = {
            let seed = seed.clone();
            Callback::from(move |e: InputEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                if let Ok(v) = input.value().trim().parse::<u64>() {
                    seed.set(v);
                }
            })
        };
        let on_name = {
            let name = name.clone();
            Callback::from(move |e: InputEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                name.set(input.value());
            })
        };
        let reroll = {
            let seed = seed.clone();
            Callback::from(move |_: MouseEvent| {
                seed.set(js_sys::Date::now() as u64);
            })
        };

        html! {
            <main class="page">
                <header class="head">
                    <h1>{ "健常な人間エミュレータ" }</h1>
                    <p class="lead">
                        { "五つの欲求と一枚の予定表だけで、人間の一週間を分単位で回す。" }
                        { "出てきた生活がどれだけ「ふつう」だったかを健常度として採点する。" }
                    </p>
                </header>

                <section class="controls">
                    <label>
                        <span>{ "被験者" }</span>
                        <input type="text" value={(*name).clone()} oninput={on_name} />
                    </label>
                    <label>
                        <span>{ "シード" }</span>
                        <input type="number" min="0" value={seed.to_string()} oninput={on_seed} />
                    </label>
                    <label>
                        <span>{ "日数" }</span>
                        <div class="chips">
                            { for DAY_CHOICES.iter().map(|&d| {
                                let days = days.clone();
                                let active = *days == d;
                                let onclick = Callback::from(move |_: MouseEvent| days.set(d));
                                html! {
                                    <button class={classes!("chip", active.then_some("on"))} {onclick}>
                                        { format!("{d}日") }
                                    </button>
                                }
                            }) }
                        </div>
                    </label>
                    <button class="reroll" onclick={reroll}>{ "別の一週間を生きる" }</button>
                </section>

                { summary(&human) }
                { day_reports(&human, &selected, selected_day) }
                { menu_of_day(&human, selected_day) }
                { timeline(&human, selected_day) }
                { time_use(&human) }
                { final_needs(&human) }

                <footer class="foot">
                    { "Rust + Yew / WebAssembly — シミュレーションはブラウザの中だけで走っている。" }
                </footer>
            </main>
        }
    }

    fn verdict_class(score: f32) -> &'static str {
        match score {
            s if s >= 85.0 => "good",
            s if s >= 70.0 => "ok",
            s if s >= 50.0 => "warn",
            _ => "bad",
        }
    }

    fn summary(h: &Human) -> Html {
        let normalcy = h.normalcy();
        html! {
            <section class="cards">
                <div class={classes!("card", verdict_class(normalcy))}>
                    <span class="card-label">{ "健常度" }</span>
                    <span class="card-value">{ format!("{normalcy:.1}") }</span>
                    <span class="card-note">{ h.verdict() }</span>
                </div>
                <div class="card">
                    <span class="card-label">{ "平均気分" }</span>
                    <span class="card-value">{ format!("{:.1}", h.average_mood()) }</span>
                    <span class="card-note">{ h.needs.mood_label() }</span>
                </div>
                <div class="card">
                    <span class="card-label">{ "総睡眠" }</span>
                    <span class="card-value">
                        { format!("{:.1}", h.days.iter().map(|d| d.sleep_minutes).sum::<u32>() as f32 / 60.0) }
                    </span>
                    <span class="card-note">{ "時間" }</span>
                </div>
                <div class="card">
                    <span class="card-label">{ "平均栄養" }</span>
                    <span class="card-value">
                        { if h.days.is_empty() { "0".to_string() } else {
                            format!("{:.0}", h.days.iter().map(|d| d.nutrition).sum::<f32>() / h.days.len() as f32)
                        } }
                    </span>
                    <span class="card-note">{ "/ 100" }</span>
                </div>
                <div class="card">
                    <span class="card-label">{ "対人接触" }</span>
                    <span class="card-value">
                        { h.days.iter().map(|d| d.social_minutes).sum::<u32>() }
                    </span>
                    <span class="card-note">{ "分" }</span>
                </div>
            </section>
        }
    }

    fn day_reports(h: &Human, selected: &UseStateHandle<u32>, current: u32) -> Html {
        html! {
            <section class="panel">
                <h2>{ "日報" }</h2>
                <table class="days">
                    <thead>
                        <tr>
                            <th>{ "日" }</th><th>{ "睡眠" }</th><th>{ "勤務" }</th>
                            <th>{ "対人" }</th><th>{ "食事" }</th><th>{ "栄養" }</th><th>{ "健常度" }</th>
                            <th class="reasons">{ "所見" }</th>
                        </tr>
                    </thead>
                    <tbody>
                    { for h.days.iter().map(|d| {
                        let selected = selected.clone();
                        let day = d.day;
                        let onclick = Callback::from(move |_: MouseEvent| selected.set(day));
                        html! {
                            <tr class={classes!((day == current).then_some("current"))} {onclick}>
                                <td>{ format!("{}日目({})", d.day + 1, d.weekday.label()) }</td>
                                <td>{ format!("{}時間{}分", d.sleep_minutes / 60, d.sleep_minutes % 60) }</td>
                                <td>{ format!("{}分", d.work_minutes) }</td>
                                <td>{ format!("{}分", d.social_minutes) }</td>
                                <td>{ format!("{}回", d.meals) }</td>
                                <td>{ format!("{:.0}", d.nutrition) }</td>
                                <td class={classes!("score", verdict_class(d.score))}>
                                    { format!("{:.0}", d.score) }
                                </td>
                                <td class="reasons">
                                    { if d.penalties.is_empty() {
                                        html! { <span class="fine">{ "問題なし" }</span> }
                                    } else {
                                        html! { <>
                                            { for d.penalties.iter().map(|(reason, cost)| html! {
                                                <span class="tag">{ format!("{reason} -{cost:.0}") }</span>
                                            }) }
                                        </> }
                                    } }
                                </td>
                            </tr>
                        }
                    }) }
                    </tbody>
                </table>
            </section>
        }
    }

    fn timeline(h: &Human, day: u32) -> Html {
        let entries: Vec<_> = h
            .merged_log()
            .into_iter()
            .filter(|e| e.at.day() == day)
            .collect();
        let label = entries
            .first()
            .map(|e| format!("{}日目({})", day + 1, e.at.weekday().label()))
            .unwrap_or_else(|| format!("{}日目", day + 1));

        html! {
            <section class="panel">
                <h2>{ format!("行動記録 — {label}") }</h2>
                <p class="hint">{ "日報の行をクリックすると、その日に切り替わる。" }</p>
                <ol class="timeline">
                { for entries.iter().map(|e| html! {
                    <li>
                        <span class="at">{ e.at.hhmm() }</span>
                        <span class={classes!("act", act_class(e.activity))}>{ e.activity.label() }</span>
                        <span class="dur">{ format!("{}分", e.minutes) }</span>
                        <span class="mood-bar">
                            <span class="mood-fill" style={format!("width:{:.0}%", e.mood)} />
                        </span>
                        <span class="remark">{ match e.meal {
                            Some((course, dish)) => format!("{}「{}」", course.label(), dish.name),
                            None => e.remark.to_string(),
                        } }</span>
                    </li>
                }) }
                </ol>
            </section>
        }
    }

    /// その日の献立。何を食べたかは健常度に直に効く。
    fn menu_of_day(h: &Human, day: u32) -> Html {
        let Some(report) = h.days.iter().find(|d| d.day == day) else {
            return Html::default();
        };
        html! {
            <section class="panel">
                <h2>{ format!("献立 — 栄養 {:.0} / 100", report.nutrition) }</h2>
                <ul class="menu">
                { for Course::ALL.iter().map(|&c| {
                    let dish = report.menu[c.index()];
                    html! {
                        <li>
                            <span class="course">{ c.label() }</span>
                            <span class="dish">
                                { dish.map(|d| d.name).unwrap_or("抜き") }
                                { if dish.is_some_and(|d| d.vegetables) {
                                    html! { <span class="veg" title="野菜あり">{ "菜" }</span> }
                                } else {
                                    Html::default()
                                } }
                            </span>
                            <span class="bar-track">
                                <span class="bar-fill nutrition"
                                      style={format!("width:{:.0}%", dish.map_or(0.0, |d| d.nutrition))} />
                            </span>
                            <span class="bar-value">{ format!("{:.0}", dish.map_or(0.0, |d| d.nutrition)) }</span>
                        </li>
                    }
                }) }
                </ul>
            </section>
        }
    }

    fn act_class(a: Activity) -> &'static str {
        match a {
            Activity::Sleep | Activity::Nap => "a-sleep",
            Activity::Work | Activity::Commute => "a-work",
            Activity::Meal | Activity::Snack => "a-meal",
            Activity::Chat | Activity::Meetup => "a-social",
            Activity::Bathe | Activity::Chores => "a-care",
            _ => "a-free",
        }
    }

    fn time_use(h: &Human) -> Html {
        let total = h.total_minutes().max(1) as f32;
        html! {
            <section class="panel">
                <h2>{ "時間の使い方" }</h2>
                <ul class="bars">
                { for h.time_by_activity().into_iter().map(|(a, m)| {
                    let share = m as f32 / total * 100.0;
                    html! {
                        <li>
                            <span class="bar-label">{ a.label() }</span>
                            <span class="bar-track">
                                <span class={classes!("bar-fill", act_class(a))}
                                      style={format!("width:{share:.2}%")} />
                            </span>
                            <span class="bar-value">{ format!("{share:.1}%") }</span>
                        </li>
                    }
                }) }
                </ul>
            </section>
        }
    }

    fn final_needs(h: &Human) -> Html {
        html! {
            <section class="panel">
                <h2>{ "終了時点の欲求" }</h2>
                <ul class="bars">
                { for Drive::ALL.iter().map(|&d| {
                    let v = h.needs.get(d);
                    html! {
                        <li>
                            <span class="bar-label">{ d.label() }</span>
                            <span class="bar-track">
                                <span class="bar-fill need" style={format!("width:{v:.1}%")} />
                            </span>
                            <span class="bar-value">{ format!("{v:.0}") }</span>
                        </li>
                    }
                }) }
                </ul>
            </section>
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    yew::Renderer::<app::App>::new().render();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("これはブラウザ向けのバイナリ。`trunk serve` で起動する。");
    eprintln!("CLI は `cargo run --bin human-emulator`。");
    std::process::exit(1);
}
