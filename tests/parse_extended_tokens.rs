use bms_rs::lex::BmsLexOutput;
use bms_rs::lex::command::ObjId;
use bms_rs::parse::BmsParseOutput;
use bms_rs::{
    lex::{LexWarning, parse},
    parse::{Bms, prompt::AlwaysWarn, rng::RngMock},
};

#[test]
fn test_atbga_parsing() {
    let source = r#"
#TITLE Test BMS
#@BGA01 02 10 20 100 200 30 40
"#;
    let BmsLexOutput {
        tokens,
        lex_warnings: warnings,
    } = parse(source);
    assert_eq!(warnings, vec![]);
    let BmsParseOutput {
        bms,
        parse_warnings,
        ..
    } = Bms::from_token_stream(&tokens, RngMock([1]), AlwaysWarn);
    assert_eq!(parse_warnings, vec![]);
    // Verify that #@BGA is parsed correctly
    assert!(
        bms.header
            .atbga_defs
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
    let atbga_def = &bms.header.atbga_defs[&ObjId::try_from(['0', '1']).unwrap()];
    assert_eq!(atbga_def.source_bmp, ObjId::try_from(['0', '2']).unwrap());
    assert_eq!(
        atbga_def.trim_top_left,
        bms_rs::parse::header::PixelPoint::new(10, 20)
    );
    assert_eq!(
        atbga_def.trim_size,
        bms_rs::parse::header::PixelSize::new(100, 200)
    );
    assert_eq!(
        atbga_def.draw_point,
        bms_rs::parse::header::PixelPoint::new(30, 40)
    );
}

#[test]
fn test_bga_parsing() {
    let source = r#"
#TITLE Test BMS
#BGA01 02 10 20 110 220 30 40
"#;
    let BmsLexOutput {
        tokens,
        lex_warnings: warnings,
    } = parse(source);
    assert_eq!(warnings, vec![]);
    let BmsParseOutput {
        bms,
        parse_warnings,
        ..
    } = Bms::from_token_stream(&tokens, RngMock([1]), AlwaysWarn);
    assert_eq!(parse_warnings, vec![]);

    // Verify that #BGA is parsed correctly
    assert!(
        bms.header
            .bga_defs
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
    let bga_def = &bms.header.bga_defs[&ObjId::try_from(['0', '1']).unwrap()];
    assert_eq!(bga_def.source_bmp, ObjId::try_from(['0', '2']).unwrap());
    assert_eq!(
        bga_def.trim_top_left,
        bms_rs::parse::header::PixelPoint::new(10, 20)
    );
    assert_eq!(
        bga_def.trim_bottom_right,
        bms_rs::parse::header::PixelPoint::new(110, 220)
    );
    assert_eq!(
        bga_def.draw_point,
        bms_rs::parse::header::PixelPoint::new(30, 40)
    );
}

#[test]
fn test_exrank_parsing() {
    let source = r#"
#TITLE Test BMS
#EXRANK01 2
"#;
    let BmsLexOutput {
        tokens,
        lex_warnings: warnings,
    } = parse(source);
    assert_eq!(warnings, vec![]);
    let BmsParseOutput {
        bms,
        parse_warnings,
        ..
    } = Bms::from_token_stream(&tokens, RngMock([1]), AlwaysWarn);
    assert_eq!(parse_warnings, vec![]);

    // Verify that #EXRANK is parsed correctly
    assert!(
        bms.header
            .exrank_defs
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
    let exrank_def = &bms.header.exrank_defs[&ObjId::try_from(['0', '1']).unwrap()];
    assert_eq!(
        exrank_def.judge_level,
        bms_rs::lex::command::JudgeLevel::Normal
    );
}

#[test]
fn test_exwav_parsing() {
    let source = r#"
#TITLE Test BMS
#EXWAV01 pvf 10000 0 48000 test.wav
"#;
    let BmsLexOutput {
        tokens,
        lex_warnings: warnings,
    } = parse(source);
    assert_eq!(warnings, vec![]);
    let BmsParseOutput {
        bms,
        parse_warnings,
        ..
    } = Bms::from_token_stream(&tokens, RngMock([1]), AlwaysWarn);
    assert_eq!(parse_warnings, vec![]);

    // Verify that #EXWAV is parsed correctly
    assert!(
        bms.header
            .exwav_defs
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
    let exwav_def = &bms.header.exwav_defs[&ObjId::try_from(['0', '1']).unwrap()];
    assert_eq!(exwav_def.pan.value(), 10000);
    assert_eq!(exwav_def.volume.value(), 0);
    assert_eq!(exwav_def.frequency.map(|f| f.value()), Some(48000));
    assert_eq!(exwav_def.path.to_string_lossy(), "test.wav");
}

#[test]
fn test_changeoption_parsing() {
    let source = r#"
#TITLE Test BMS
#CHANGEOPTION01 test_option
"#;
    let BmsLexOutput {
        tokens,
        lex_warnings: warnings,
    } = parse(source);
    assert_eq!(warnings, vec![]);
    let BmsParseOutput {
        bms,
        parse_warnings,
        ..
    } = Bms::from_token_stream(&tokens, RngMock([1]), AlwaysWarn);
    assert_eq!(parse_warnings, vec![]);

    // Verify that #CHANGEOPTION is parsed correctly
    assert!(
        bms.header
            .change_options
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
    let option = &bms.header.change_options[&ObjId::try_from(['0', '1']).unwrap()];
    assert_eq!(option, "test_option");
}

#[test]
fn test_text_parsing() {
    let source = r#"
#TITLE Test BMS
#TEXT01 test_text
"#;
    let BmsLexOutput {
        tokens,
        lex_warnings: warnings,
    } = parse(source);
    assert_eq!(warnings, vec![]);
    let BmsParseOutput {
        bms,
        parse_warnings,
        ..
    } = Bms::from_token_stream(&tokens, RngMock([1]), AlwaysWarn);
    assert_eq!(parse_warnings, vec![]);

    // Verify that #TEXT is parsed correctly
    assert!(
        bms.header
            .texts
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
    let text = &bms.header.texts[&ObjId::try_from(['0', '1']).unwrap()];
    assert_eq!(text, "test_text");
}

#[test]
fn test_notes_parse_extended_tokens() {
    let source = r#"
#TITLE Test BMS
#EXRANK01 2
#EXWAV01 pvf 10000 0 48000 test.wav
#EXWAV02 vpf 0 10000 48000 test2.wav
#CHANGEOPTION01 test_option
#TEXT01 test_text
"#;
    let BmsLexOutput {
        tokens,
        lex_warnings: warnings,
    } = parse(source);
    assert_eq!(warnings, vec![]);
    let BmsParseOutput {
        bms,
        parse_warnings,
        ..
    } = Bms::from_token_stream(&tokens, RngMock([1]), AlwaysWarn);
    assert_eq!(parse_warnings, vec![]);

    // Verify that extended fields in Notes are parsed correctly
    assert!(
        bms.notes
            .exrank_defs
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
    assert!(
        bms.notes
            .exwav_defs
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
    assert!(
        bms.notes
            .exwav_defs
            .contains_key(&ObjId::try_from(['0', '2']).unwrap())
    );
    assert!(
        bms.notes
            .change_options
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
    assert!(
        bms.notes
            .texts
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
}

#[test]
fn test_token_parsing_comprehensive() {
    let source = r#"
#TITLE Test BMS
#ARTIST Test Artist
#EMAIL test@example.com
#URL http://example.com
#MAKER Test Maker
#MIDIFILE test.mid
#VIDEOFILE test.mp4
#POORBGA 1
#OCT/FP
#PATH_WAV wav/
#@BGA01 02 10 20 100 200 30 40
#BGA02 03 15 25 150 250 35 45
#EXRANK01 2
#EXWAV01 pvf 10000 0 48000 test.wav
#CHANGEOPTION01 test_option
#TEXT01 test_text
"#;
    let BmsLexOutput {
        tokens,
        lex_warnings: warnings,
    } = parse(source);
    assert_eq!(warnings, vec![]);
    let BmsParseOutput {
        bms,
        parse_warnings,
        ..
    } = Bms::from_token_stream(&tokens, RngMock([1]), AlwaysWarn);
    assert_eq!(parse_warnings, vec![]);

    // Verify that all new tokens are parsed correctly
    assert_eq!(bms.header.artist, Some("Test Artist".to_string()));
    assert_eq!(bms.header.email, Some("test@example.com".to_string()));
    assert_eq!(bms.header.url, Some("http://example.com".to_string()));
    assert_eq!(bms.header.maker, Some("Test Maker".to_string()));
    assert_eq!(
        bms.header.midi_file,
        Some(std::path::PathBuf::from("test.mid"))
    );
    assert_eq!(
        bms.header.video_file,
        Some(std::path::PathBuf::from("test.mp4"))
    );
    assert_eq!(
        bms.header.poor_bga_mode,
        bms_rs::lex::command::PoorMode::Overlay
    );
    assert!(bms.header.is_octave);
    assert_eq!(
        bms.header.wav_path_root,
        Some(std::path::PathBuf::from("wav/"))
    );

    // Verify new definition structures
    assert!(
        bms.header
            .atbga_defs
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
    assert!(
        bms.header
            .bga_defs
            .contains_key(&ObjId::try_from(['0', '2']).unwrap())
    );
    assert!(
        bms.header
            .exrank_defs
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
    assert!(
        bms.header
            .exwav_defs
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
    assert!(
        bms.header
            .change_options
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
    assert!(
        bms.header
            .texts
            .contains_key(&ObjId::try_from(['0', '1']).unwrap())
    );
}

#[test]
fn test_exwav_out_of_range_values() {
    // Test pan value out of range
    let source = r#"
#TITLE Test BMS
#EXWAV01 p 10001 test.wav
"#;
    let BmsLexOutput {
        tokens,
        lex_warnings: warnings,
    } = parse(source);
    let [warn] = &warnings[..] else {
        panic!("expected 1 warning, got: {warnings:?}");
    };
    match warn {
        LexWarning::ExpectedToken { message, .. }
            if message.starts_with("pan value out of range") => {}
        other => panic!("unexpected warning type: {other:?}"),
    }
    let BmsParseOutput {
        bms: _,
        parse_warnings,
        ..
    } = Bms::from_token_stream(&tokens, RngMock([1]), AlwaysWarn);
    assert_eq!(parse_warnings, vec![]);

    // Test volume value out of range
    let source = r#"
#TITLE Test BMS
#EXWAV01 v 1 test.wav
"#;
    let BmsLexOutput {
        tokens,
        lex_warnings: warnings,
    } = parse(source);
    let [warn] = &warnings[..] else {
        panic!("expected 1 warning, got: {warnings:?}");
    };
    match warn {
        LexWarning::ExpectedToken { message, .. }
            if message.starts_with("volume value out of range") => {}
        other => panic!("unexpected warning type: {other:?}"),
    }
    let BmsParseOutput {
        bms: _,
        parse_warnings,
        ..
    } = Bms::from_token_stream(&tokens, RngMock([1]), AlwaysWarn);
    assert_eq!(parse_warnings, vec![]);

    // Test frequency value out of range
    let source = r#"
#TITLE Test BMS
#EXWAV01 f 99 test.wav
"#;
    let BmsLexOutput {
        tokens,
        lex_warnings: warnings,
    } = parse(source);
    let [warn] = &warnings[..] else {
        panic!("expected 1 warning, got: {warnings:?}");
    };
    match warn {
        LexWarning::ExpectedToken { message, .. }
            if message.starts_with("frequency value out of range") => {}
        other => panic!("unexpected warning type: {other:?}"),
    }
    let BmsParseOutput {
        bms: _,
        parse_warnings,
        ..
    } = Bms::from_token_stream(&tokens, RngMock([1]), AlwaysWarn);
    assert_eq!(parse_warnings, vec![]);
}
