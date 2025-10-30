//! This module provides [`TokenProcessor`] and its implementations, which reads [`Token`] and applies data to [`Bms`].
//!
//! Also it provides preset functions that returns a [`TokenProcessor`] trait object:
//!
//! - [`common_preset`] - Commonly used processors.
//! - [`minor_preset`] - All of processors this crate provided.

use std::{borrow::Cow, cell::RefCell, num::NonZeroU64, rc::Rc};

use itertools::Itertools;

use crate::bms::{
    error::{ParseError, ParseErrorWithRange},
    prelude::*,
};

mod bmp;
mod bpm;
mod identity;
mod judge;
mod metadata;
mod music_info;
mod option;
mod random;
mod repr;
mod resources;
mod scroll;
mod section_len;
mod speed;
mod sprite;
mod stop;
mod text;
mod video;
mod volume;
mod wav;

/// A type alias of `Result<(T, Vec<ParseWarningWithRange>), ParseErrorWithRange>`.
pub type TokenProcessorResult<T> = Result<(T, Vec<ParseWarningWithRange>), ParseErrorWithRange>;

/// A processor of tokens in the BMS. An implementation takes control only one feature about definitions and placements such as `WAVxx` definition and its sound object.
pub trait TokenProcessor {
    /// A result data of the process.
    type Output;

    /// Processes commands by consuming all the stream `input`. It mutates `input`
    fn process<P: Prompter>(
        &self,
        input: &mut &[&TokenWithRange<'_>],
        prompter: &P,
    ) -> TokenProcessorResult<Self::Output>;

    /// Creates a processor [`SequentialProcessor`] which does `self` then `second`.
    fn then<S>(self, second: S) -> SequentialProcessor<Self, S>
    where
        Self: Sized,
        S: TokenProcessor + Sized,
    {
        SequentialProcessor {
            first: self,
            second,
        }
    }

    /// Maps a result of the processor by the mapping function `f`.
    fn map<F, O>(self, f: F) -> Mapped<Self, F>
    where
        Self: Sized,
        F: Fn((Self::Output, Vec<ParseWarningWithRange>)) -> (O, Vec<ParseWarningWithRange>),
    {
        Mapped {
            source: self,
            mapping: f,
        }
    }
}

impl<T: TokenProcessor + ?Sized> TokenProcessor for Box<T> {
    type Output = <T as TokenProcessor>::Output;

    fn process<P: Prompter>(
        &self,
        input: &mut &[&TokenWithRange<'_>],
        prompter: &P,
    ) -> TokenProcessorResult<Self::Output> {
        T::process(self, input, prompter)
    }
}

/// A processor [`SequentialProcessor`] which does `first` then `second`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequentialProcessor<F, S> {
    first: F,
    second: S,
}

impl<F, S> TokenProcessor for SequentialProcessor<F, S>
where
    F: TokenProcessor,
    S: TokenProcessor,
{
    type Output = (F::Output, S::Output);

    fn process<P: Prompter>(
        &self,
        input: &mut &[&TokenWithRange<'_>],
        prompter: &P,
    ) -> TokenProcessorResult<Self::Output> {
        let mut cloned = *input;
        let (first_output, mut first_warnings) = self.first.process(&mut cloned, prompter)?;
        let (second_output, second_warnings) = self.second.process(input, prompter)?;
        first_warnings.extend(second_warnings);
        Ok(((first_output, second_output), first_warnings))
    }
}

/// A processor [`SequentialProcessor`] which maps the output of the token processor `TP` by the function `F`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mapped<TP, F> {
    source: TP,
    mapping: F,
}

impl<O, TP, F> TokenProcessor for Mapped<TP, F>
where
    TP: TokenProcessor,
    F: Fn((TP::Output, Vec<ParseWarningWithRange>)) -> (O, Vec<ParseWarningWithRange>),
{
    type Output = O;

    fn process<P: Prompter>(
        &self,
        input: &mut &[&TokenWithRange<'_>],
        prompter: &P,
    ) -> TokenProcessorResult<Self::Output> {
        let (res, warnings) = self.source.process(input, prompter)?;
        Ok(self.mapping((res, warnings)))
    }
}

/// Returns commonly used processors.
pub(crate) fn common_preset<T: KeyLayoutMapper, R: Rng>(
    rng: Rc<RefCell<R>>,
    relaxed: bool,
) -> impl TokenProcessor<Output = Bms> {
    let case_sensitive_obj_id = Rc::new(RefCell::new(false));
    let sub_processor = repr::RepresentationProcessor::new(&case_sensitive_obj_id)
        .then(bmp::BmpProcessor::new(&case_sensitive_obj_id))
        .then(bpm::BpmProcessor::new(&case_sensitive_obj_id))
        .then(judge::JudgeProcessor::new(&case_sensitive_obj_id))
        .then(metadata::MetadataProcessor)
        .then(music_info::MusicInfoProcessor)
        .then(scroll::ScrollProcessor::new(&case_sensitive_obj_id))
        .then(section_len::SectionLenProcessor)
        .then(speed::SpeedProcessor::new(&case_sensitive_obj_id))
        .then(sprite::SpriteProcessor)
        .then(stop::StopProcessor::new(&case_sensitive_obj_id))
        .then(video::VideoProcessor::new(&case_sensitive_obj_id))
        .then(wav::WavProcessor::<T>::new(&case_sensitive_obj_id));
    random::RandomTokenProcessor::new(rng, sub_processor, relaxed).map(
        |(
            (
                (
                    (
                        (
                            (
                                ((((((repr, bmp), bpm), judge), metadata), music_info), scroll),
                                section_len,
                            ),
                            speed,
                        ),
                        sprite,
                    ),
                    stop,
                ),
                video,
            ),
            wav,
        ), warnings| {
            (Bms {
                bmp,
                bpm,
                judge,
                metadata,
                music_info,

                option: Default::default(),
                repr,

                resources: Default::default(),
                scroll,
                section_len,
                speed,
                sprite,
                stop,
                text: Default::default(),
                video,
                volume: Default::default(),
                wav,
            }, warnings)
        },
    )
}

/// Returns all of processors this crate provided.
pub(crate) fn minor_preset<T: KeyLayoutMapper, R: Rng>(
    rng: Rc<RefCell<R>>,
    relaxed: bool,
) -> impl TokenProcessor<Output = Bms> {
    let case_sensitive_obj_id = Rc::new(RefCell::new(false));
    let sub_processor = repr::RepresentationProcessor::new(&case_sensitive_obj_id)
        .then(bmp::BmpProcessor::new(&case_sensitive_obj_id))
        .then(bpm::BpmProcessor::new(&case_sensitive_obj_id))
        .then(judge::JudgeProcessor::new(&case_sensitive_obj_id))
        .then(metadata::MetadataProcessor)
        .then(music_info::MusicInfoProcessor);

    let sub_processor = sub_processor
        .then(option::OptionProcessor::new(&case_sensitive_obj_id))
        .then(resources::ResourcesProcessor);
    let sub_processor = sub_processor
        .then(scroll::ScrollProcessor::new(&case_sensitive_obj_id))
        .then(section_len::SectionLenProcessor)
        .then(speed::SpeedProcessor::new(&case_sensitive_obj_id))
        .then(sprite::SpriteProcessor)
        .then(stop::StopProcessor::new(&case_sensitive_obj_id))
        .then(text::TextProcessor::new(&case_sensitive_obj_id))
        .then(video::VideoProcessor::new(&case_sensitive_obj_id))
        .then(volume::VolumeProcessor)
        .then(wav::WavProcessor::<T>::new(&case_sensitive_obj_id));
    random::RandomTokenProcessor::new(rng, sub_processor, relaxed).map(
        |(
            (
                (
                    (
                        (
                            (
                                (
                                    (
                                        (
                                            (
                                                (
                                                    (
                                                        ((((repr, bmp), bpm), judge), metadata),
                                                        music_info,
                                                    ),
                                                    option,
                                                ),
                                                resources,
                                            ),
                                            scroll,
                                        ),
                                        section_len,
                                    ),
                                    speed,
                                ),
                                sprite,
                            ),
                            stop,
                        ),
                        text,
                    ),
                    video,
                ),
                volume,
            ),
            wav,
        ), warnings| (Bms {
            bmp,
            bpm,
            judge,
            metadata,
            music_info,
            option,
            repr,
            resources,
            scroll,
            section_len,
            speed,
            sprite,
            stop,
            text,
            video,
            volume,
            wav,
        }, warnings),
    )
}

fn all_tokens<
    'a,
    P: Prompter,
    F: FnMut(&'a Token<'_>) -> Result<(Option<ParseWarning>, Vec<ParseWarningWithRange>), ParseError>,
>(
    input: &mut &'a [&TokenWithRange<'_>],
    prompter: &P,
    mut f: F,
) -> TokenProcessorResult<()> {
    let mut warnings = Vec::new();
    for token in &**input {
        let (maybe_warning, mut extra_warnings) = f(token.content()).map_err(|err| err.into_wrapper(token))?;
        warnings.append(&mut extra_warnings);
        if let Some(warning) = maybe_warning {
            warnings.push(warning.into_wrapper(token));
        }
    }
    *input = &[];
    Ok(((), warnings))
}

fn all_tokens_with_range<
    'a,
    P: Prompter,
    F: FnMut(&'a TokenWithRange<'_>) -> Result<(Option<ParseWarning>, Vec<ParseWarningWithRange>), ParseError>,
>(
    input: &mut &'a [&TokenWithRange<'_>],
    prompter: &P,
    mut f: F,
) -> TokenProcessorResult<()> {
    let mut warnings = Vec::new();
    for token in &**input {
        let (maybe_warning, mut extra_warnings) = f(token).map_err(|err| err.into_wrapper(token))?;
        warnings.append(&mut extra_warnings);
        if let Some(warning) = maybe_warning {
            warnings.push(warning.into_wrapper(token));
        }
    }
    *input = &[];
    Ok(((), warnings))
}

fn parse_obj_ids(
    track: Track,
    message: SourceRangeMixin<&str>,
    case_sensitive_obj_id: &RefCell<bool>,
) -> (Vec<ParseWarningWithRange>, impl Iterator<Item = (ObjTime, ObjId)>) {
    let mut warnings = Vec::new();
    if !message.content().len().is_multiple_of(2) {
        warnings.push(
            ParseWarning::SyntaxError("expected 2-digit object ids".into()).into_wrapper(&message),
        );
    }

    let denom_opt = NonZeroU64::new(message.content().len() as u64 / 2);
    let iterator = message
        .content()
        .chars()
        .tuples()
        .enumerate()
        .filter_map(move |(i, (c1, c2))| {
            let buf = String::from_iter([c1, c2]);
            match ObjId::try_from(&buf, *case_sensitive_obj_id.borrow()) {
                Ok(id) if id.is_null() => None,
                Ok(id) => Some((
                    ObjTime::new(
                        track.0,
                        i as u64,
                        denom_opt.expect("len / 2 won't be zero on reading tuples"),
                    ),
                    id,
                )),
                Err(warning) => {
                    warnings.push(warning.into_wrapper(&message));
                    None
                }
            }
        });
    (warnings, iterator)
}

fn parse_hex_values(
    track: Track,
    message: SourceRangeMixin<&str>,
) -> (Vec<ParseWarningWithRange>, impl Iterator<Item = (ObjTime, u8)>) {
    let mut warnings = Vec::new();
    if !message.content().len().is_multiple_of(2) {
        warnings.push(
            ParseWarning::SyntaxError("expected 2-digit hex values".into()).into_wrapper(&message),
        );
    }

    let denom_opt = NonZeroU64::new(message.content().len() as u64 / 2);
    let iterator = message
        .content()
        .chars()
        .tuples()
        .enumerate()
        .filter_map(move |(i, (c1, c2))| {
            let buf = String::from_iter([c1, c2]);
            match u8::from_str_radix(&buf, 16) {
                Ok(value) => Some((
                    ObjTime::new(
                        track.0,
                        i as u64,
                        denom_opt.expect("len / 2 won't be zero on reading tuples"),
                    ),
                    value,
                )),
                Err(_) => {
                    warnings.push(
                        ParseWarning::SyntaxError(format!("invalid hex digits ({buf:?}"))
                            .into_wrapper(&message),
                    );
                    None
                }
            }
        });
    (warnings, iterator)
}

fn filter_message(message: &str) -> Cow<'_, str> {
    let result = message
        .chars()
        .try_fold(String::with_capacity(message.len()), |mut acc, ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '.' {
                acc.push(ch);
                Ok(acc)
            } else {
                Err(acc)
            }
        });
    match result {
        Ok(_) => Cow::Borrowed(message),
        Err(filtered) => Cow::Owned(filtered),
    }
}
