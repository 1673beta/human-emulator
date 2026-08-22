//! 行動と気分から、当たり障りのない独り言を生成する。
//! 健常な人間の発話はおおむね予測可能で、内容がない。

use crate::activity::Activity;
use crate::rng::Rng;

/// 候補から一つ選ぶ。
fn one(rng: &mut Rng, xs: &[&'static str]) -> &'static str {
    // 明示的な参照外し。外すと T が str に推論されてしまう。
    #[allow(clippy::explicit_auto_deref)]
    *rng.pick(xs)
}

pub fn remark(activity: Activity, mood: f32, rng: &mut Rng) -> &'static str {
    if mood < 30.0 {
        return one(
            rng,
            &[
                "……ちょっと限界かも。",
                "今日はもう早めに寝よう。",
                "うまく回らないな。",
            ],
        );
    }

    match activity {
        Activity::Sleep => one(rng, &["おやすみ。", "明日も早いし寝るか。", "もう寝よう。"]),
        Activity::Nap => one(rng, &["少しだけ横になろう。", "十五分だけ……。"]),
        Activity::Meal => one(
            rng,
            &[
                "いただきます。",
                "今日は何食べようかな。",
                "ちゃんと食べておこう。",
            ],
        ),
        Activity::Snack => one(rng, &["小腹すいたな。", "ひと息つこう。"]),
        Activity::Bathe => one(rng, &["さっぱりした。", "風呂入るか。"]),
        Activity::Commute => one(
            rng,
            &["いつもの電車。", "今日も混んでるな。", "そろそろ出るか。"],
        ),
        Activity::Work => {
            if mood >= 70.0 {
                one(
                    rng,
                    &[
                        "よし、片付けるか。",
                        "順調に進んでる。",
                        "午後もこの調子で。",
                    ],
                )
            } else {
                one(
                    rng,
                    &[
                        "ぼちぼちやるか。",
                        "早く終わらないかな。",
                        "とりあえず手を動かす。",
                    ],
                )
            }
        }
        Activity::Chat => one(
            rng,
            &[
                "返信しておかないと。",
                "そういえば連絡きてたな。",
                "元気にしてるかな。",
            ],
        ),
        Activity::Meetup => one(rng, &["久しぶりだな。", "会えてよかった。", "楽しかった。"]),
        Activity::Exercise => one(rng, &["体動かしておくか。", "少し走ってこよう。"]),
        Activity::Chores => one(
            rng,
            &[
                "洗濯物溜まってる。",
                "片付けるか。",
                "ついでに掃除もしよう。",
            ],
        ),
        Activity::Hobby => one(rng, &["この時間がいちばんいい。", "続きやろう。"]),
        Activity::Idle => one(
            rng,
            &["…………。", "特にやることないな。", "なんとなく時間が過ぎる。"],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 台詞は必ず返る() {
        let mut rng = Rng::new(1);
        for a in Activity::VOLUNTARY {
            assert!(!remark(a, 70.0, &mut rng).is_empty());
        }
        assert!(!remark(Activity::Work, 70.0, &mut rng).is_empty());
        assert!(!remark(Activity::Commute, 70.0, &mut rng).is_empty());
    }

    #[test]
    fn 気分が悪いと弱音を吐く() {
        let mut rng = Rng::new(3);
        let s = remark(Activity::Work, 10.0, &mut rng);
        assert!(s.contains("限界") || s.contains("寝よう") || s.contains("回らない"));
    }
}
