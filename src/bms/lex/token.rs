//! Definitions of the token in BMS format.

use std::{borrow::Cow, path::Path};

use super::{Result, command::*, cursor::Cursor};

/// A token of BMS format.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub enum Token<'a> {
    /// `#ARTIST [string]`. Defines the artist name of the music.
    Artist(&'a str),
    /// `#@BGA[01-ZZ] [01-ZZ] [sx] [sy] [w] [h] [dx] [dy]`. Defines the image object from trimming the existing image object.
    AtBga {
        /// The id of the object to define.
        id: ObjId,
        /// The id of the object to be trimmed.
        source_bmp: ObjId,
        /// The top left point of the trim area in pixels.
        trim_top_left: (i16, i16),
        /// The size of the trim area in pixels.
        trim_size: (u16, u16),
        /// The top left point to be rendered in pixels.
        draw_point: (i16, i16),
    },
    /// `#BANNER [filename]`. Defines the banner image. This can be used on music select or result view. It should be 300x80.
    Banner(&'a Path),
    /// `#BACKBMP [filename]`. Defines the background image file of the play view. It should be 640x480. The effect will depend on the skin of the player.
    BackBmp(&'a Path),
    /// `#BASE 62`. Declares that the score is using base-62 object id format. If this exists, the score is treated as case-sensitive.
    Base62,
    /// `#BGA[01-ZZ] [01-ZZ] [x1] [y1] [x2] [y2] [dx] [dy]`. Defines the image object from trimming the existing image object.
    Bga {
        /// The id of the object to define.
        id: ObjId,
        /// The id of the object to be trimmed.
        source_bmp: ObjId,
        /// The top left point of the trim area in pixels.
        trim_top_left: (i16, i16),
        /// The bottom right point of the trim area in pixels.
        trim_bottom_right: (i16, i16),
        /// The top left point to be rendered in pixels.
        draw_point: (i16, i16),
    },
    /// `#BMP[01-ZZ] [filename]`. Defines the background image/movie object. The file specified may be not only BMP format, and also PNG, AVI, MP4, MKV and others. Its size should be less than or equal to 256x256. The black (`#000000`) pixel in the image will be treated as transparent. When the id `00` is specified, this first field will be `None` and the image will be shown when the player get mistaken.
    Bmp(Option<ObjId>, &'a Path),
    /// `#BPM [f64]`. Defines the base Beats-Per-Minute of the score. Defaults to 130, but some players don't conform to it.
    Bpm(&'a str),
    /// `#BPM[01-ZZ] [f64]`. Defines the Beats-Per-Minute change object.
    BpmChange(ObjId, &'a str),
    /// `#CASE [u32]`. Starts a case scope if the integer equals to the generated random number. If there's no `#SKIP` command in the scope, the parsing will **fallthrough** to the next `#CASE` or `#DEF`. See also [`Token::Switch`].
    Case(u32),
    /// `#CHANGEOPTION[01-ZZ] [string]`. Defines the play option change object. Some players interpret and apply the preferences.
    ChangeOption(ObjId, &'a str),
    /// `#COMMENT [string]`. Defines the text which is shown in the music select view. This may or may not be surrounded by double-quotes.
    Comment(&'a str),
    /// `#DEF`. Starts a case scope if any `#CASE` had not matched to the generated random number. It must be placed in the end of the switch scope. See also [`Token::Switch`].
    Def,
    /// `#DIFFICULTY [1-5]`. Defines the difficulty of the score. It can be used to sort the score having the same title.
    Difficulty(u8),
    /// `#ELSEIF [u32]`. Starts an if scope when the preceding `#IF` had not matched to the generated random number. It must be in an if scope.
    Else,
    /// `#ELSEIF [u32]`. Starts an if scope when the integer equals to the generated random number. It must be in an if scope. If preceding `#IF` had matched to the generated, this scope don't start. Syntax sugar for:
    ///
    /// ```text
    /// #ELSE
    ///   #IF n
    ///   // ...
    ///   #ENDIF
    /// #ENDIF
    /// ```
    ElseIf(u32),
    /// `%EMAIL [string]`. The email address of this score file author.
    Email(&'a str),
    /// `#ENDIF`. Closes the if scope. See [Token::If].
    EndIf,
    /// `#ENDRANDOM`. Closes the random scope. See [Token::Random].
    EndRandom,
    /// `#ENDSW`. Closes the random scope. See [Token::Switch].
    EndSwitch,
    /// `#EXT #XXXYY:...`. Defines the extended message. `XXX` is the track, `YY` is the channel.
    ExtendedMessage {
        /// The track, or measure, must start from 1. But some player may allow the 0 measure (i.e. Lunatic Rave 2).
        track: Track,
        /// The channel commonly expresses what the lane be arranged the note to.
        channel: Channel,
        /// The message to the channel, but not only object ids.
        message: &'a str,
    },
    /// `#BMP[01-ZZ] [0-255],[0-255],[0-255],[0-255] [filename]`. Defines the background image/movie object with the color (alpha, red, green and blue) which will be treated as transparent.
    ExBmp(ObjId, Argb, &'a Path),
    /// `#EXRANK[01-ZZ] [0-3]`. Defines the judgement level change object.
    ExRank(ObjId, JudgeLevel),
    /// `#EXWAV[01-ZZ] [parameter order] [pan or volume or frequency; 1-3] [filename]`. Defines the key sound object with the effect of pan, volume and frequency.
    ExWav {
        /// The id of the object to define.
        id: ObjId,
        /// The pan decay of the sound. Also called volume balance.
        pan: ExWavPan,
        /// The volume decay of the sound.
        volume: ExWavVolume,
        /// The pitch frequency of the sound.
        frequency: Option<ExWavFrequency>,
        /// The relative file path of the sound.
        path: &'a Path,
    },
    /// `#GENRE [string]`. Defines the genre of the music.
    Genre(&'a str),
    /// `#IF [u32]`. Starts an if scope when the integer equals to the generated random number. This must be placed in a random scope. See also [`Token::Random`].
    If(u32),
    /// `#LNOBJ [01-ZZ]`. Declares the object as the end of an LN. The preceding object of the declared will be treated as the beginning of an LN.
    LnObj(ObjId),
    /// `#LNTYPE 1`. Declares the LN notation as the RDM type.
    LnTypeRdm,
    /// `#LNTYPE 2`. Declares the LN notation as the MGQ type.
    LnTypeMgq,
    /// `#MAKER [string]`. Defines the author name of the score.
    Maker(&'a str),
    /// `#XXXYY:ZZ...`. Defines the message which places the object onto the score. `XXX` is the track, `YY` is the channel, and `ZZ...` is the object id sequence.
    Message {
        /// The track, or measure, must start from 1. But some player may allow the 0 measure (i.e. Lunatic Rave 2).
        track: Track,
        /// The channel commonly expresses what the lane be arranged the note to.
        channel: Channel,
        /// The message to the channel.
        message: Cow<'a, str>,
    },
    /// `#MIDIFILE [filename]`. Defines the MIDI file as the BGM. *Deprecated*
    MidiFile(&'a Path),
    /// Non-empty lines that not starts in `'#'` in bms file.
    NotACommand(&'a str),
    /// `#OCT/FP`. Declares the score as the octave mode.
    OctFp,
    /// `#OPTION [string]`. Defines the play option of the score. Some players interpret and apply the preferences.
    Option(&'a str),
    /// `#PATH_WAV [string]`. Defines the root path of [`Token::Wav`] paths. This should be used only for tests.
    PathWav(&'a Path),
    /// `#PLAYER [1-4]`. Defines the play style of the score.
    Player(PlayerMode),
    /// `#PLAYLEVEL [integer]`. Defines the difficulty level of the score. This can be used on music select view.
    PlayLevel(u8),
    /// `#POORBGA [0-2]`. Defines the display mode of the POOR BGA.
    PoorBga(PoorMode),
    /// `#RANDOM [u32]`. Starts a random scope which can contain only `#IF`-`#ENDIF` scopes. The random scope must close with `#ENDRANDOM`. A random integer from 1 to the integer will be generated when parsing the score. Then if the integer of `#IF` equals to the random integer, the commands in an if scope will be parsed, otherwise all command in it will be ignored. Any command except `#IF` and `#ENDIF` must not be included in the scope, but some players allow it.
    Random(u32),
    /// `#RANK [0-3]`. Defines the judgement level.
    Rank(JudgeLevel),
    /// `#SCROLL[01-ZZ] [f64]`. Defines the scroll speed change object. It changes relative falling speed of notes with keeping BPM. For example, if applying `2.0`, the scroll speed will become double.
    Scroll(ObjId, &'a str),
    /// `#SETRANDOM [u32]`. Starts a random scope but the integer will be used as the generated random number. It should be used only for tests.
    SetRandom(u32),
    /// `#SETSWITCH [u32]`. Starts a switch scope but the integer will be used as the generated random number. It should be used only for tests.
    SetSwitch(u32),
    /// `#SKIP`. Escapes the current switch scope. It is often used in the end of every case scope.
    Skip,
    /// `#SPEED[01-ZZ] [f64]`. Defines the spacing change object. It changes relative spacing of notes with linear interpolation. For example, if playing score between the objects `1.0` and `2.0`, the spaces of notes will increase at the certain rate until the `2.0` object.
    Speed(ObjId, &'a str),
    /// `#STAGEFILE [filename]`. Defines the splashscreen image. It should be 640x480.
    StageFile(&'a Path),
    /// `#STOP[01-ZZ] [0-4294967295]`. Defines the stop object. The scroll will stop the beats of the integer divided by 192. A beat length depends on the current BPM. If there are other objects on same time, the stop object must be evaluated at last.
    Stop(ObjId, u32),
    /// `#SUBARTIST [string]`. Defines the sub-artist name of the music.
    SubArtist(&'a str),
    /// `#SUBTITLE [string]`. Defines the subtitle of the music.
    SubTitle(&'a str),
    /// `#SWITCH [u32]`. Starts a switch scope which can contain only `#CASE` or `#DEF` scopes. The switch scope must close with `#ENDSW`. A random integer from 1 to the integer will be generated when parsing the score. Then if the integer of `#CASE` equals to the random integer, the commands in a case scope will be parsed, otherwise all command in it will be ignored. Any command except `#CASE` and `#DEF` must not be included in the scope, but some players allow it.
    Switch(u32),
    /// `#TEXT[01-ZZ] string`. Defines the text object.
    Text(ObjId, &'a str),
    /// `#TITLE [string]`. Defines the title of the music.
    Title(&'a str),
    /// `#TOTAL [f64]`. Defines the total gauge percentage when all notes is got as PERFECT.
    Total(&'a str),
    /// Unknown Part. Includes all the line that not be parsed.
    UnknownCommand(&'a str),
    /// `%URL [string]`. The url of this score file.
    Url(&'a str),
    /// `#VIDEOFILE [filename]` / `#MOVIE [filename]`. Defines the background movie file. The audio track in the movie file should not be played. The play should start from the track `000`.
    VideoFile(&'a Path),
    /// `#VOLWAV [0-255]`. Defines the relative volume percentage of the sound in the score.
    VolWav(Volume),
    /// `#WAV[01-ZZ] [filename]`. Defines the key sound object. When same id multiple objects ring at same time, it must be played only one. The file specified may be not only WAV format, and also OGG, MP3 and others.
    Wav(ObjId, &'a Path),
}

impl<'a> Token<'a> {
    pub(crate) fn parse(
        c: &mut Cursor<'a>,
        channel_parser: impl Fn(&str) -> Option<Channel>,
    ) -> Result<Self> {
        loop {
            let command = c
                .next_token()
                .ok_or_else(|| c.make_err_expected_token("command"))?;

            break Ok(match command.to_uppercase().as_str() {
                // Part: Normal
                "#PLAYER" => Self::Player(PlayerMode::from(c)?),
                "#GENRE" => Self::Genre(c.next_line_remaining()),
                "#TITLE" => Self::Title(c.next_line_remaining()),
                "#SUBTITLE" => Self::SubTitle(c.next_line_remaining()),
                "#ARTIST" => Self::Artist(c.next_line_remaining()),
                "#SUBARTIST" => Self::SubArtist(c.next_line_remaining()),
                "#DIFFICULTY" => Self::Difficulty(
                    c.next_token()
                        .ok_or_else(|| c.make_err_expected_token("difficulty"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?,
                ),
                "#STAEGFILE" => {
                    let file_name = c.next_line_remaining();
                    if file_name.is_empty() {
                        return Err(c.make_err_expected_token("stage filename"));
                    }
                    Self::StageFile(Path::new(file_name))
                }
                "#BANNER" => {
                    let file_name = c.next_line_remaining();
                    if file_name.is_empty() {
                        return Err(c.make_err_expected_token("banner filename"));
                    }
                    Self::Banner(Path::new(file_name))
                }
                "#BACKBMP" => {
                    let file_name = c.next_line_remaining();
                    if file_name.is_empty() {
                        return Err(c.make_err_expected_token("backbmp filename"));
                    }
                    Self::BackBmp(Path::new(file_name))
                }
                "#TOTAL" => Self::Total(
                    c.next_token()
                        .ok_or_else(|| c.make_err_expected_token("gauge increase rate"))?,
                ),
                "#BPM" => Self::Bpm(
                    c.next_token()
                        .ok_or_else(|| c.make_err_expected_token("bpm"))?,
                ),
                "#PLAYLEVEL" => Self::PlayLevel(
                    c.next_token()
                        .ok_or_else(|| c.make_err_expected_token("play level"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?,
                ),
                "#RANK" => Self::Rank(JudgeLevel::try_read(c)?),
                "#LNTYPE" => {
                    if c.next_token() == Some("2") {
                        Self::LnTypeMgq
                    } else {
                        Self::LnTypeRdm
                    }
                }
                // Part: ControlFlow/Random
                "#RANDOM" => {
                    let rand_max = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("random max"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    Self::Random(rand_max)
                }
                "#SETRANDOM" => {
                    let rand_value = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("random value"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    Self::SetRandom(rand_value)
                }
                "#IF" => {
                    let rand_target = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("random target"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    Self::If(rand_target)
                }
                "#ELSEIF" => {
                    let rand_target = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("random target"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    Self::ElseIf(rand_target)
                }
                "#ELSE" => Self::Else,
                "#ENDIF" => Self::EndIf,
                "#ENDRANDOM" => Self::EndRandom,
                // Part: ControlFlow/Switch
                "#SWITCH" => {
                    let switch_max = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("switch max"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    Self::Switch(switch_max)
                }
                "#SETSWITCH" => {
                    let switch_value = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("switch value"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    Self::SetSwitch(switch_value)
                }
                "#CASE" => {
                    let case_value = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("switch case value"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    Self::Case(case_value)
                }
                "#SKIP" => Self::Skip,
                "#DEF" => Self::Def, // See https://hitkey.bms.ms/cmds.htm#DEF
                "#ENDSW" => Self::EndSwitch, // See https://hitkey.bms.ms/cmds.htm#ENDSW
                // Part: Normal 2
                "#STAGEFILE" => {
                    let file_name = c.next_line_remaining();
                    if file_name.is_empty() {
                        return Err(c.make_err_expected_token("splashscreen image filename"));
                    }
                    Self::StageFile(Path::new(file_name))
                }
                "#VOLWAV" => {
                    let volume = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("volume"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    Self::VolWav(Volume {
                        relative_percent: volume,
                    })
                }
                "#BASE" => {
                    let base = c.next_line_remaining();
                    if base != "62" {
                        eprintln!("unknown base declared: {base:?}");
                        continue;
                    }
                    Self::Base62
                }
                "#COMMENT" => {
                    let comment = c.next_line_remaining();
                    Self::Comment(comment)
                }
                "#EMAIL" | "%EMAIL" => Self::Email(c.next_line_remaining()),
                "#URL" | "%URL" => Self::Url(c.next_line_remaining()),
                "#OCT/FP" => Self::OctFp,
                "#OPTION" => Self::Option(c.next_line_remaining()),
                "#PATH_WAV" => {
                    let file_name = c.next_line_remaining();
                    if file_name.is_empty() {
                        return Err(c.make_err_expected_token("wav root path"));
                    }
                    Self::PathWav(Path::new(file_name))
                }
                "#MAKER" => Self::Maker(c.next_line_remaining()),
                "#MIDIFILE" => {
                    let file_name = c.next_line_remaining();
                    if file_name.is_empty() {
                        return Err(c.make_err_expected_token("midi filename"));
                    }
                    Self::MidiFile(Path::new(file_name))
                }
                "#POORBGA" => Self::PoorBga(PoorMode::from(c)?),
                "#VIDEOFILE" | "#MOVIE" => {
                    let file_name = c.next_line_remaining();
                    if file_name.is_empty() {
                        return Err(c.make_err_expected_token("video filename"));
                    }
                    Self::VideoFile(Path::new(file_name))
                }
                // Part: Command with lane and arg
                wav if wav.starts_with("#WAV") => {
                    let id = command.trim_start_matches("#WAV");
                    let str = c.next_line_remaining();
                    if str.is_empty() {
                        return Err(c.make_err_expected_token("key audio filename"));
                    }
                    let filename = Path::new(str);
                    Self::Wav(ObjId::try_load(id, c)?, filename)
                }
                bmp if bmp.starts_with("#BMP") => {
                    let id = command.trim_start_matches("#BMP");
                    let str = c.next_line_remaining();
                    if str.is_empty() {
                        return Err(c.make_err_expected_token("key audio filename"));
                    }
                    let filename = Path::new(str);
                    if id == "00" {
                        Self::Bmp(None, filename)
                    } else {
                        Self::Bmp(Some(ObjId::try_load(id, c)?), filename)
                    }
                }
                bpm if bpm.starts_with("#BPM") => {
                    let id = command.trim_start_matches("#BPM");
                    let bpm = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("bpm"))?;
                    Self::BpmChange(ObjId::try_load(id, c)?, bpm)
                }
                stop if stop.starts_with("#STOP") => {
                    let id = command.trim_start_matches("#STOP");
                    let stop = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("stop beats"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    Self::Stop(ObjId::try_load(id, c)?, stop)
                }
                scroll if scroll.starts_with("#SCROLL") => {
                    let id = command.trim_start_matches("#SCROLL");
                    let scroll = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("scroll factor"))?;
                    Self::Scroll(ObjId::try_load(id, c)?, scroll)
                }
                speed if speed.starts_with("#SPEED") => {
                    let id = command.trim_start_matches("#SPEED");
                    let scroll = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("spacing factor"))?;
                    Self::Speed(ObjId::try_load(id, c)?, scroll)
                }
                exbmp if exbmp.starts_with("#EXBMP") => {
                    let id = exbmp.trim_start_matches("#EXBMP");
                    let argb = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("argb"))?;
                    let filename = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("filename"))?;

                    let parts: Vec<&str> = argb.split(',').collect();
                    if parts.len() != 4 {
                        return Err(c.make_err_expected_token("expected 4 comma-separated values"));
                    }
                    let alpha = parts[0]
                        .parse()
                        .map_err(|_| c.make_err_expected_token("invalid alpha value"))?;
                    let red = parts[1]
                        .parse()
                        .map_err(|_| c.make_err_expected_token("invalid red value"))?;
                    let green = parts[2]
                        .parse()
                        .map_err(|_| c.make_err_expected_token("invalid green value"))?;
                    let blue = parts[3]
                        .parse()
                        .map_err(|_| c.make_err_expected_token("invalid blue value"))?;

                    Self::ExBmp(
                        ObjId::try_load(id, c)?,
                        Argb {
                            alpha,
                            red,
                            green,
                            blue,
                        },
                        Path::new(filename),
                    )
                }
                exrank if exrank.starts_with("#EXRANK") => {
                    let id = exrank.trim_start_matches("#EXRANK");
                    let judge_level = JudgeLevel::try_read(c)?;
                    Self::ExRank(ObjId::try_load(id, c)?, judge_level)
                }
                exwav if exwav.starts_with("#EXWAV") => {
                    let id = exwav.trim_start_matches("#EXWAV");
                    let pvf_params = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("param1"))?;
                    let mut pan = None;
                    let mut volume = None;
                    let mut frequency = None;
                    for param in pvf_params.bytes() {
                        match param {
                            b'p' => {
                                let pan_value: i64 = c
                                    .next_token()
                                    .ok_or_else(|| c.make_err_expected_token("pan"))?
                                    .parse()
                                    .map_err(|_| c.make_err_expected_token("integer"))?;
                                pan = Some(ExWavPan::try_from(pan_value).map_err(|_| {
                                    c.make_err_expected_token(
                                        "pan value out of range [-10000, 10000]",
                                    )
                                })?)
                            }
                            b'v' => {
                                let volume_value: i64 = c
                                    .next_token()
                                    .ok_or_else(|| c.make_err_expected_token("volume"))?
                                    .parse()
                                    .map_err(|_| c.make_err_expected_token("integer"))?;
                                volume =
                                    Some(ExWavVolume::try_from(volume_value).map_err(|_| {
                                        c.make_err_expected_token(
                                            "volume value out of range [-10000, 0]",
                                        )
                                    })?)
                            }
                            b'f' => {
                                let frequency_value: u64 = c
                                    .next_token()
                                    .ok_or_else(|| c.make_err_expected_token("frequency"))?
                                    .parse()
                                    .map_err(|_| c.make_err_expected_token("integer"))?;
                                frequency = Some(
                                    ExWavFrequency::try_from(frequency_value).map_err(|_| {
                                        c.make_err_expected_token(
                                            "frequency value out of range [100, 100000]",
                                        )
                                    })?,
                                )
                            }
                            _ => return Err(c.make_err_expected_token("expected p, v or f")),
                        }
                    }
                    let file_name = c.next_line_remaining();
                    if file_name.is_empty() {
                        return Err(c.make_err_expected_token("filename"));
                    }
                    Self::ExWav {
                        id: ObjId::try_load(id, c)?,
                        pan: pan.unwrap_or_default(),
                        volume: volume.unwrap_or_default(),
                        frequency,
                        path: Path::new(file_name),
                    }
                }
                text if text.starts_with("#TEXT") => {
                    let id = text.trim_start_matches("#TEXT");
                    let content = c.next_line_remaining();
                    Self::Text(ObjId::try_load(id, c)?, content)
                }
                atbga if atbga.starts_with("#@BGA") => {
                    let id = atbga.trim_start_matches("#@BGA");
                    let source_bmp = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("source bmp"))?;
                    let sx = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("sx"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    let sy = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("sy"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    let w = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("w"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    let h = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("h"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    let dx = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("dx"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    let dy = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("dy"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    Self::AtBga {
                        id: ObjId::try_load(id, c)?,
                        source_bmp: ObjId::try_load(source_bmp, c)?,
                        trim_top_left: (sx, sy),
                        trim_size: (w, h),
                        draw_point: (dx, dy),
                    }
                }
                bga if bga.starts_with("#BGA") && !bga.starts_with("#BGAPOOR") => {
                    let id = bga.trim_start_matches("#BGA");
                    // Cannot use next_line_remaining here because the remaining args
                    let source_bmp = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("source bmp"))?;
                    let x1 = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("x1"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    let y1 = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("y1"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    let x2 = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("x2"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    let y2 = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("y2"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    let dx = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("dx"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    let dy = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("dy"))?
                        .parse()
                        .map_err(|_| c.make_err_expected_token("integer"))?;
                    Self::Bga {
                        id: ObjId::try_load(id, c)?,
                        source_bmp: ObjId::try_load(source_bmp, c)?,
                        trim_top_left: (x1, y1),
                        trim_bottom_right: (x2, y2),
                        draw_point: (dx, dy),
                    }
                }
                changeoption if changeoption.starts_with("#CHANGEOPTION") => {
                    let id = changeoption.trim_start_matches("#CHANGEOPTION");
                    let option = c.next_line_remaining();
                    Self::ChangeOption(ObjId::try_load(id, c)?, option)
                }
                lnobj if lnobj.starts_with("#LNOBJ") => {
                    let id = lnobj.trim_start_matches("#LNOBJ");
                    Self::LnObj(ObjId::try_load(id, c)?)
                }
                ext_message if ext_message.starts_with("#EXT") => {
                    let message = c
                        .next_token()
                        .ok_or_else(|| c.make_err_expected_token("message definition"))?;
                    if !(message.starts_with('#')
                        && message.chars().nth(6) == Some(':')
                        && 8 <= message.len())
                    {
                        eprintln!("unknown #EXT format: {message:?}");
                        continue;
                    }

                    let track = message[1..4]
                        .parse()
                        .map_err(|_| c.make_err_expected_token("[000-999]"))?;
                    let channel = &message[4..6];
                    let message = &message[7..];
                    Self::ExtendedMessage {
                        track: Track(track),
                        channel: channel_parser(channel)
                            .ok_or_else(|| c.make_err_unknown_channel(channel.to_string()))?,
                        message,
                    }
                }
                message
                    if message.starts_with('#')
                        && message.chars().nth(6) == Some(':')
                        && 8 <= message.len() =>
                {
                    let track = command[1..4]
                        .parse()
                        .map_err(|_| c.make_err_expected_token("[000-999]"))?;
                    let channel = &command[4..6];

                    let message = &command[7..];
                    Self::Message {
                        track: Track(track),
                        channel: channel_parser(channel)
                            .ok_or_else(|| c.make_err_unknown_channel(channel.to_string()))?,
                        message: Cow::Borrowed(message),
                    }
                }
                command if command.starts_with('#') => Self::UnknownCommand(c.next_line_entire()),
                _not_command => Self::NotACommand(c.next_line_entire()),
            });
        }
    }

    pub(crate) fn make_id_uppercase(&mut self) {
        use Token::*;
        match self {
            AtBga { id, source_bmp, .. } => {
                id.make_uppercase();
                source_bmp.make_uppercase();
            }
            Bga { id, source_bmp, .. } => {
                id.make_uppercase();
                source_bmp.make_uppercase();
            }
            Bmp(Some(id), _) => {
                id.make_uppercase();
            }
            BpmChange(id, _) => {
                id.make_uppercase();
            }
            ChangeOption(id, _) => {
                id.make_uppercase();
            }
            ExBmp(id, _, _) => {
                id.make_uppercase();
            }
            ExRank(id, _) => {
                id.make_uppercase();
            }
            ExWav { id, .. } => {
                id.make_uppercase();
            }
            LnObj(id) => {
                id.make_uppercase();
            }
            Message { message, .. } => {
                if message.chars().any(|ch| ch.is_ascii_lowercase()) {
                    message.to_mut().make_ascii_uppercase();
                }
            }
            Scroll(id, _) => {
                id.make_uppercase();
            }
            Speed(id, _) => {
                id.make_uppercase();
            }
            Stop(id, _) => {
                id.make_uppercase();
            }
            Text(id, _) => {
                id.make_uppercase();
            }
            Wav(id, _) => {
                id.make_uppercase();
            }
            _ => {}
        }
    }

    /// Checks if a token is a control flow token.
    pub fn is_control_flow_token(&self) -> bool {
        matches!(
            self,
            Token::Random(_)
                | Token::SetRandom(_)
                | Token::If(_)
                | Token::ElseIf(_)
                | Token::Else
                | Token::EndIf
                | Token::EndRandom
                | Token::Switch(_)
                | Token::SetSwitch(_)
                | Token::Case(_)
                | Token::Def
                | Token::Skip
                | Token::EndSwitch
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::lex::command::channel::read_channel_beat;

    use super::*;

    fn parse_token(input: &str) -> Token {
        let mut cursor = Cursor::new(input);
        Token::parse(&mut cursor, read_channel_beat).unwrap()
    }

    #[test]
    fn test_exbmp() {
        let Token::ExBmp(id, argb, path) = parse_token("#EXBMP01 255,0,0,0 exbmp.png") else {
            panic!("Not ExBmp");
        };
        assert_eq!(format!("{id:?}"), "ObjId(\"01\")");
        assert_eq!(argb.alpha, 255);
        assert_eq!(argb.red, 0);
        assert_eq!(argb.green, 0);
        assert_eq!(argb.blue, 0);
        assert_eq!(path, Path::new("exbmp.png"));
    }

    #[test]
    fn test_exrank() {
        let Token::ExRank(id, level) = parse_token("#EXRANK01 2") else {
            panic!("Not ExRank");
        };
        assert_eq!(format!("{id:?}"), "ObjId(\"01\")");
        assert_eq!(level, JudgeLevel::Normal);
    }

    #[test]
    fn test_exwav() {
        let Token::ExWav {
            id,
            pan,
            volume,
            frequency,
            path: file,
        } = parse_token("#EXWAV01 pvf 10000 0 48000 ex.wav")
        else {
            panic!("Not ExWav");
        };
        assert_eq!(format!("{id:?}"), "ObjId(\"01\")");
        assert_eq!(pan.value(), 10000);
        assert_eq!(volume.value(), 0);
        assert_eq!(frequency.map(|f| f.value()), Some(48000));
        assert_eq!(file, Path::new("ex.wav"));
    }

    #[test]
    fn test_exwav_2() {
        let Token::ExWav {
            id,
            pan,
            volume,
            frequency,
            path: file,
        } = parse_token("#EXWAV01 vpf 0 10000 48000 ex.wav")
        else {
            panic!("Not ExWav");
        };
        assert_eq!(format!("{id:?}"), "ObjId(\"01\")");
        assert_eq!(pan.value(), 10000);
        assert_eq!(volume.value(), 0);
        assert_eq!(frequency.map(|f| f.value()), Some(48000));
        assert_eq!(file, Path::new("ex.wav"));
    }

    #[test]
    fn test_exwav_default() {
        let Token::ExWav {
            id,
            pan,
            volume,
            frequency,
            path: file,
        } = parse_token("#EXWAV01 f 48000 ex.wav")
        else {
            panic!("Not ExWav");
        };
        assert_eq!(format!("{id:?}"), "ObjId(\"01\")");
        assert_eq!(pan.value(), 0);
        assert_eq!(volume.value(), 0);
        assert_eq!(frequency.map(|f| f.value()), Some(48000));
        assert_eq!(file, Path::new("ex.wav"));
    }

    #[test]
    fn test_text() {
        let Token::Text(id, text) = parse_token("#TEXT01 hello world") else {
            panic!("Not Text");
        };
        assert_eq!(format!("{id:?}"), "ObjId(\"01\")");
        assert_eq!(text, "hello world");
    }

    #[test]
    fn test_atbga() {
        let Token::AtBga {
            id,
            source_bmp,
            trim_top_left,
            trim_size,
            draw_point,
        } = parse_token("#@BGA01 02 1 2 3 4 5 6")
        else {
            panic!("Not AtBga");
        };
        assert_eq!(format!("{id:?}"), "ObjId(\"01\")");
        assert_eq!(format!("{source_bmp:?}"), "ObjId(\"02\")");
        assert_eq!(trim_top_left, (1, 2));
        assert_eq!(trim_size, (3, 4));
        assert_eq!(draw_point, (5, 6));
    }

    #[test]
    fn test_bga() {
        let Token::Bga {
            id,
            source_bmp,
            trim_top_left,
            trim_bottom_right,
            draw_point,
        } = parse_token("#BGA01 02 1 2 3 4 5 6")
        else {
            panic!("Not Bga");
        };
        assert_eq!(format!("{id:?}"), "ObjId(\"01\")");
        assert_eq!(format!("{source_bmp:?}"), "ObjId(\"02\")");
        assert_eq!(trim_top_left, (1, 2));
        assert_eq!(trim_bottom_right, (3, 4));
        assert_eq!(draw_point, (5, 6));
    }

    #[test]
    fn test_changeoption() {
        let Token::ChangeOption(id, opt) = parse_token("#CHANGEOPTION01 opt") else {
            panic!("Not ChangeOption");
        };
        assert_eq!(format!("{id:?}"), "ObjId(\"01\")");
        assert_eq!(opt, "opt");
    }

    #[test]
    fn test_lnobj() {
        let Token::LnObj(id) = parse_token("#LNOBJ01") else {
            panic!("Not LnObj");
        };
        assert_eq!(format!("{id:?}"), "ObjId(\"01\")");
    }
}
