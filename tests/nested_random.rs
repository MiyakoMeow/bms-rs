use bms_rs::{
    lex::{
        command::{Key, NoteKind},
        parse,
    },
    parse::{obj::Obj, rng::RngMock, Bms},
    time::ObjTime,
};

#[test]
fn nested_random() {
    const SRC: &str = r"
        #00111:11000000

        #RANDOM 2

        #IF 1
            #00112:00220000

            #RANDOM 2

            #IF 1
                #00115:00550000
            #ENDIF

            #IF 2
                #00116:00006600
            #ENDIF

            #ENDRANDOM

        #ENDIF

        #IF 2
            #00113:00003300
        #ENDIF

        #ENDRANDOM

        #00114:00000044
    ";

    let id11 = 37.try_into().unwrap();
    let id22 = 74.try_into().unwrap();
    let id33 = 111.try_into().unwrap();
    let id44 = 148.try_into().unwrap();
    let id55 = 185.try_into().unwrap();
    let id66 = 222.try_into().unwrap();

    let ts = parse(SRC).expect("must be parsed");
    let rng = RngMock([1]);
    let bms = Bms::from_token_stream(&ts, rng).expect("must be parsed");
    assert_eq!(
        bms.notes.into_all_notes(),
        vec![
            Obj {
                offset: ObjTime::new(1, 0, 4),
                kind: NoteKind::Visible,
                is_player1: true,
                key: Key::Key1,
                obj: id11,
            },
            Obj {
                offset: ObjTime::new(1, 1, 4),
                kind: NoteKind::Visible,
                is_player1: true,
                key: Key::Key2,
                obj: id22,
            },
            Obj {
                offset: ObjTime::new(1, 1, 4),
                kind: NoteKind::Visible,
                is_player1: true,
                key: Key::Key5,
                obj: id55,
            },
            Obj {
                offset: ObjTime::new(1, 3, 4),
                kind: NoteKind::Visible,
                is_player1: true,
                key: Key::Key4,
                obj: id44,
            }
        ]
    );

    let rng = RngMock([1, 2]);
    let bms = Bms::from_token_stream(&ts, rng).expect("must be parsed");
    assert_eq!(
        bms.notes.into_all_notes(),
        vec![
            Obj {
                offset: ObjTime::new(1, 0, 4),
                kind: NoteKind::Visible,
                is_player1: true,
                key: Key::Key1,
                obj: id11,
            },
            Obj {
                offset: ObjTime::new(1, 1, 4),
                kind: NoteKind::Visible,
                is_player1: true,
                key: Key::Key2,
                obj: id22,
            },
            Obj {
                offset: ObjTime::new(1, 2, 4),
                kind: NoteKind::Visible,
                is_player1: true,
                key: Key::Scratch,
                obj: id66,
            },
            Obj {
                offset: ObjTime::new(1, 3, 4),
                kind: NoteKind::Visible,
                is_player1: true,
                key: Key::Key4,
                obj: id44,
            }
        ]
    );

    let rng = RngMock([2]);
    let bms = Bms::from_token_stream(&ts, rng).expect("must be parsed");
    assert_eq!(
        bms.notes.into_all_notes(),
        vec![
            Obj {
                offset: ObjTime::new(1, 0, 4),
                kind: NoteKind::Visible,
                is_player1: true,
                key: Key::Key1,
                obj: id11,
            },
            Obj {
                offset: ObjTime::new(1, 2, 4),
                kind: NoteKind::Visible,
                is_player1: true,
                key: Key::Key3,
                obj: id33,
            },
            Obj {
                offset: ObjTime::new(1, 3, 4),
                kind: NoteKind::Visible,
                is_player1: true,
                key: Key::Key4,
                obj: id44,
            }
        ]
    );
}
