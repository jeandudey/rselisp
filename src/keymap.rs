// Copyright (C) 2017 Richard Palethorpe <richiejp@f-m.fm>

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use rselisp::{Lsp, Inner, LispForm, Sexp, Func, EvalOption};
use editor::{Event, BasicEvent, EventModifiers};
use fnv::FnvHashMap;
use std::any::Any;
use std::slice::Iter;

#[derive(Debug)]
pub struct Keymap {
    map: FnvHashMap<Event, Inner>,
}

impl Keymap {
    pub fn new() -> Keymap {
        Keymap {
            map: FnvHashMap::default(),
        }
    }

    pub fn define_key(&mut self, key: Event, def: Inner) {
        self.map.insert(key, def);
    }

    pub fn parse_key(&mut self, key: &str) -> Result<Event, String> {
        let mut mods = EventModifiers::new();
        let mut basic: Option<BasicEvent> = None;
        let mut itr = key.chars();

        while let Some(c) = itr.next() {
            match c {
                '\\' => {
                    match itr.next() {
                        Some('C') => mods.control = true,
                        Some('S') => mods.shift = true,
                        Some('M') => mods.alt = true,
                        Some('\\') => {
                            basic = Some(BasicEvent::Char('\\'));
                            break;
                        }
                        Some(c) => return Err(format!("Unrecognised escape character: {}", c)),
                        _ => return Err(format!("Unexpected end of string after \\")),
                    };
                    if Some('-') != itr.next() {
                        return Err(format!("Expecting '-' after modifier"));
                    }
                },
                c => {
                    basic = Some(BasicEvent::Char(c));
                    break;
                }
            }
        }

        if basic.is_none() {
            Err(format!("Key string does not contain a key"))
        } else if itr.next().is_some() {
            Err(format!("Unexpected characters at end of key string"))
        } else {
            Ok(Event::new(basic.unwrap(), mods))
        }
    }

    pub fn lookup_key(&self, key: &Event) -> Option<&Inner> {
        self.map.get(key)
    }

    pub fn is_keymap(obj: &LispForm) -> bool {
        obj.lisp_name() == "keymap"
    }
}

impl LispForm for Keymap {
    fn rust_name(&self) -> &'static str {
        "keymap::Keymap"
    }

    fn lisp_name(&self) -> &'static str {
        "keymap"
    }

    fn to_lisp(&self) -> Result<Inner, String> {
        let mut sxp = Sexp::from(&[Inner::sym("keymap")]);

        for (evt, act) in self.map.iter() {
            sxp.push(Inner::pair(evt.to_lisp()?, act.clone()));
        }

        Ok(Inner::Sxp(sxp))
    }

    fn as_any(&mut self) -> &mut Any {
        self
    }
}

def_builtin! { "keymapp", KeymapBuiltin, Evaluated, _lsp, args; {
    if let Some(s) = args.next() {
        match s {
            &Inner::Ext(ref ext) if Keymap::is_keymap(&*ext.borrow()) => Ok(Inner::t()),
            &Inner::Sxp(ref sxp) if sxp.car() == Inner::sym("keymap") => Ok(Inner::t()),
            _ => Ok(Inner::nil()),
        }
    } else {
        Err(format!("keymapp requires one argument"))
    }
}}

def_builtin! { "define-key", DefineKeyBuiltin, Evaluated, _lsp, args; {
    if let (Some(keymap), Some(evt), Some(act)) = take3!(args) {
        Ok(with_downcast!(keymap, Keymap; {
            let evt = match evt {
                &Inner::Ext(_) => with_downcast!(evt, Event; { evt.clone() } )?,
                &Inner::Str(ref s) => keymap.parse_key(s)?,
                _ => return Err(format!("Expected event string or external Event type")),
            };

            if let &Inner::Ref(_) = act {
                keymap.define_key(evt.clone(), act.clone());
            } else {
                keymap.define_key(evt.clone(), Inner::Ref(act.clone().into_ref()));
            }

            keymap.lookup_key(&evt).unwrap().clone()
        })?)
    } else {
        Err(format!("define-key requires more arguments"))
    }
}}