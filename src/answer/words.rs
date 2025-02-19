use crate::utils::constant::{ALL_NATIVE_ANSWER, ANSWER_PROBABILITY, CALL_FOR_HELP};
use crate::utils::word::filter_native_words;
use crate::utils::morpheme::{WordData, Loan, PrefixData};
use rand::{thread_rng, Rng};
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::{Requester, ResponseResult};
use teloxide::types::Message;
use teloxide::{respond, Bot};

fn build_answer_text(non_native_words: Vec<WordData>) -> String {
    if non_native_words.is_empty() {
        return ALL_NATIVE_ANSWER.to_string();
    }

    format!(
        "{}\n{}",
        non_native_words.iter().fold("".to_string(), |acc, word| acc
            + format!("{}\n", word).as_str()),
        CALL_FOR_HELP
    )
}

pub async fn words_answer(bot: Bot, msg: Message, words: Vec<Loan>, prefixes: Vec<PrefixData>) -> ResponseResult<()> {
    let msg_text = match msg.text() {
        Some(b) => b,
        None => return respond(()),
    };

    let non_native_words = filter_native_words(&words, &prefixes, msg_text.to_string());
    let is_private_chat = !msg.chat.is_group() && !msg.chat.is_supergroup();

    match (non_native_words.is_empty(), is_private_chat) {
        (_, true) => {
            bot.send_message(msg.chat.id, build_answer_text(non_native_words))
                .reply_to_message_id(msg.id)
                .await?;
            respond(())
        }
        (true, false) => respond(()),
        (false, false) => {
            if thread_rng().gen_range(0.0..1.0) <= ANSWER_PROBABILITY {
                return respond(());
            }

            bot.send_message(msg.chat.id, build_answer_text(non_native_words))
                .reply_to_message_id(msg.id)
                .await?;
            respond(())
        }
    }
}

#[cfg(test)]
mod answer_tests {
    use crate::answer::words::build_answer_text;
    use crate::utils::constant::{ALL_NATIVE_ANSWER, CALL_FOR_HELP, ROOTS_FILE};
    use crate::utils::parse::parse_from_json_file;
    use crate::utils::word::filter_native_words;
    use crate::utils::morpheme::{Loan, PrefixData};

    #[test]
    fn test_build_answer_text() {
        let words: Vec<Loan> = parse_from_json_file(ROOTS_FILE);
        let prefixes = vec![PrefixData::default()];
        let non_native_words_one = filter_native_words(&words, &prefixes, "приплясывание".to_string());

        assert_eq!(
            build_answer_text(non_native_words_one),
            ALL_NATIVE_ANSWER.to_string()
        );

        let non_native_words_two =
            filter_native_words(&words, &prefixes, "кант систематик фабричный".to_string());

        assert_eq!(
            build_answer_text(non_native_words_two),
            format!(
                "{}\n{}",
                "Если вы имели в виду не род мыслителя Иммануила Канта, то будет правильно 1) оторочка, тесьма, выпушка 2) края скользяка (у лыж и снегоката)\nНе систематик, а упорядочиватель.\nНе фабричный, а заводской.\n", CALL_FOR_HELP
            )
        );
    }
}
