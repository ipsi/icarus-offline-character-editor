use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

use directories::BaseDirs;
use druid::{AppLauncher, Data, Env, Lens, LensExt, Widget, WidgetExt, WindowDesc};
use druid::im::vector::Vector;
use druid::text::ParseFormatter;
use druid::widget::{Align, Button, Checkbox, CrossAxisAlignment, Flex, Label, LabelText, TabInfo, Tabs, TabsPolicy, TextBox, ValueTextBox, ViewSwitcher};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

const DEFAULT_INVENTORY: &'static str = "{
    \"ID\": \"MetaInventoryID_Main\",
    \"Delta\": []
}";
const TALENTS_RAW: &'static str = include_str!("talents.txt");
const BLUEPRINTS_RAW: &'static str = include_str!("blueprints.txt");
const PROSPECTS_RAW: &'static str = include_str!("prospects.txt");
const WORKSHOP_ITEMS_RAW: &'static str = include_str!("workshop_items.txt");

const EXOTIC_MINING_FLAG: f64 = 17.0;
const EXOTIC_EXTRACTION_FLAG: f64 = 18.0;

lazy_static! {
    static ref TALENT_LEVELS: HashMap<&'static str, f64> = build_map(TALENTS_RAW);
    static ref TALENTS: HashSet<&'static str> = build_set(TALENTS_RAW);
    static ref BLUEPRINTS: HashSet<&'static str> = build_set(BLUEPRINTS_RAW);
    static ref PROSPECTS: HashSet<&'static str> = build_set(PROSPECTS_RAW);
    static ref WORKSHOP_ITEMS: HashSet<&'static str> = build_set(WORKSHOP_ITEMS_RAW);
}

fn build_map(str: &'static str) -> HashMap<&'static str, f64> {
    let mut map = HashMap::<&'static str, f64>::new();
    for line in str.split("\n").into_iter() {
        let parts = line.split(",").collect::<Vec<&'static str>>();
        if parts.len() != 2 {
            panic!("Unable to parse file - expected [{}] to split into 2, but got [{:?}] instead", line, parts);
        }
        let talent_name = parts[0];
        let talent_max_level = if let Ok(parsed) = f64::from_str(parts[1]) {
            parsed
        } else {
            panic!("Unable to parse [{}] as f64", parts[1])
        };
        map.insert(talent_name, talent_max_level);
    }
    map
}

fn build_set(str: &'static str) -> HashSet<&'static str> {
    let mut set = HashSet::<&'static str>::new();
    for line in str.split("\n").into_iter() {
        let parts = line.split(",").collect::<Vec<&'static str>>();
        if parts.len() != 2 {
            panic!("Unable to parse file - expected [{}] to split into 2, but got [{:?}] instead", line, parts);
        }
        let talent_name = parts[0];
        set.insert(talent_name);
    }
    set
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Data, Lens)]
struct Characters {
    #[serde(rename = "Characters.json")]
    pub characters_json: Vector<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Data, Lens)]
struct Character {
    #[serde(rename = "CharacterName")]
    pub character_name: String,
    #[serde(rename = "ChrSlot")]
    pub character_slot: f64,
    #[serde(rename = "XP")]
    pub xp: f64,
    #[serde(rename = "XP_Debt")]
    pub xp_debt: f64,
    #[serde(rename = "IsDead")]
    pub is_dead: bool,
    #[serde(rename = "IsAbandoned")]
    pub is_abandoned: bool,
    #[serde(rename = "LastProspectId")]
    pub last_prospect_id: String,
    #[serde(rename = "Location")]
    pub location: String,
    #[serde(rename = "UnlockedFlags")]
    pub unlocked_flags: Vector<f64>,
    #[serde(rename = "MetaResources")]
    pub meta_resources: Vector<MetaResources>,
    #[serde(rename = "Cosmetic")]
    pub cosmetics: Cosmetics,
    #[serde(rename = "Talents")]
    pub talents: Vector<Talent>,
    #[data(eq)]
    #[serde(skip)]
    inventory_path: PathBuf,
    #[data(eq)]
    #[serde(skip)]
    loadout_path: PathBuf,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Data, Lens)]
struct MetaResources {
    #[serde(rename = "MetaRow")]
    pub meta_row: String,
    #[serde(rename = "Count")]
    pub count: f64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Data, Lens)]
struct Cosmetics {
    #[serde(rename = "Customization_Head")]
    customization_head: f64,
    #[serde(rename = "Customization_Hair")]
    customization_hair: f64,
    #[serde(rename = "Customization_HairColor")]
    customization_hair_color: f64,
    #[serde(rename = "Customization_Body")]
    customization_body: f64,
    #[serde(rename = "Customization_BodyColor")]
    customization_body_color: f64,
    #[serde(rename = "Customization_SkinTone")]
    customization_skin_tone: f64,
    #[serde(rename = "Customization_HeadTattoo")]
    customization_head_tattoo: f64,
    #[serde(rename = "Customization_HeadScar")]
    customization_head_scar: f64,
    #[serde(rename = "Customization_HeadFacialHair")]
    customization_head_facial_hair: f64,
    #[serde(rename = "Customization_CapLogo")]
    customization_cap_logo: f64,
    #[serde(rename = "IsMale")]
    is_male: bool,
    #[serde(rename = "Customization_Voice")]
    customization_voice: f64,
    #[serde(rename = "Customization_EyeColor")]
    customization_eye_color: f64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Data, Lens)]
struct Talent {
    #[serde(rename = "RowName")]
    pub row_name: String,
    #[serde(rename = "Rank")]
    pub rank: f64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Data, Lens)]
struct Profile {
    #[serde(rename = "UserID")]
    pub user_id: String,
    #[serde(rename = "MetaResources")]
    pub meta_resources: Vector<MetaResources>,
    #[serde(rename = "UnlockedFlags")]
    pub unlocked_flags: Vector<f64>,
    #[serde(rename = "Talents")]
    pub talents: Vector<Talent>,
}

impl Profile {
    fn unlock_all_prospects(&mut self) {
        self.talents.retain(|t| !PROSPECTS.contains(t.row_name.as_str()));
        for t in PROSPECTS.iter() {
            self.talents.push_back(Talent{row_name: (*t).to_owned(), rank: 1.0 })
        }
    }

    fn unlock_all_workshop_items(&mut self) {
        self.talents.retain(|t| !WORKSHOP_ITEMS.contains(t.row_name.as_str()));
        for t in WORKSHOP_ITEMS.iter() {
            self.talents.push_back(Talent{row_name: (*t).to_owned(), rank: 1.0 })
        }
    }
}

struct Credit();

impl Lens<Vector<MetaResources>, MetaResources> for Credit {
    fn with<V, F: FnOnce(&MetaResources) -> V>(&self, data: &Vector<MetaResources>, f: F) -> V {
        if let Some(credits) = data.iter().find(|x| x.meta_row == "Credits") {
            f(credits)
        } else {
            unimplemented!()
        }
    }

    fn with_mut<V, F: FnOnce(&mut MetaResources) -> V>(&self, data: &mut Vector<MetaResources>, f: F) -> V {
        if let Some(credits) = data.iter_mut().find(|x| x.meta_row == "Credits") {
            f(credits)
        } else {
            data.push_back(MetaResources {
                meta_row: "Credits".to_string(),
                count: 0.0,
            });
            f(data.back_mut().expect("Cannot be empty as we just pushed an object into the vec..."))
        }
    }
}

struct Exotic();

impl Lens<Vector<MetaResources>, MetaResources> for Exotic {
    fn with<V, F: FnOnce(&MetaResources) -> V>(&self, data: &Vector<MetaResources>, f: F) -> V {
        if let Some(exotics) = data.iter().find(|x| x.meta_row == "Exotic1") {
            f(exotics)
        } else {
             f(&MetaResources{ meta_row: "".to_string(), count: 0.0 })
        }
    }

    fn with_mut<V, F: FnOnce(&mut MetaResources) -> V>(&self, data: &mut Vector<MetaResources>, f: F) -> V {
        if let Some(exotics) = data.iter_mut().find(|x| x.meta_row == "Exotic1") {
            f(exotics)
        } else {
            data.push_back(MetaResources {
                meta_row: "Exotic1".to_string(),
                count: 0.0,
            });
            f(data.back_mut().expect("Cannot be empty as we just pushed an object into the vec..."))
        }
    }
}

struct FlagLens {
    flag: f64,
}

impl Lens<Vector<f64>, bool> for FlagLens {
    fn with<V, F: FnOnce(&bool) -> V>(&self, data: &Vector<f64>, f: F) -> V {
        f(&data.contains(&self.flag))
    }

    fn with_mut<V, F: FnOnce(&mut bool) -> V>(&self, data: &mut Vector<f64>, f: F) -> V {
        let mut flag = data.contains(&self.flag);
        let v = f(&mut flag);
        if flag {
            if !data.contains(&self.flag) { data.push_back(self.flag) };
        } else {
            data.retain(|x| *x != self.flag);
        }

        v
    }
}

impl Character {
    fn level_to_max(&mut self) {
        self.xp = 99_999_999.0;
    }

    fn reset_talents(&mut self) {
        self.talents.retain(|t| !TALENTS.contains(t.row_name.as_str()));
    }

    fn reset_blueprints(&mut self) {
        self.talents.retain(|t| !BLUEPRINTS.contains(t.row_name.as_str()));
    }

    fn unlock_all_talents(&mut self) {
        self.talents.retain(|t| !TALENTS.contains(t.row_name.as_str()));
        for t in TALENT_LEVELS.iter() {
            self.talents.push_back(Talent{row_name: (*t.0).to_owned(), rank: (*t.1) })
        }
    }

    fn unlock_all_blueprints(&mut self) {
        self.talents.retain(|t| !BLUEPRINTS.contains(t.row_name.as_str()));
        for t in BLUEPRINTS.iter() {
            self.talents.push_back(Talent{row_name: (*t).to_owned(), rank: 1.0 })
        }
    }

    fn restore(&mut self) -> Result<(), Box<dyn Error>> {
        self.is_abandoned = false;
        self.is_dead = false;

        self.update_inventory()?;
        self.update_loadout()?;

        Ok(())
    }

    fn update_loadout(&self) -> Result<(), Box<dyn Error>> {
        let mut file_io = OpenOptions::new().write(true).read(true).open(self.loadout_path.clone())?;
        let mut file_contents = String::new();
        file_io.read_to_string(&mut file_contents)?;

        let mut key_values: HashMap<String, serde_json::Value> = serde_json::from_str(&file_contents)?;
        key_values.insert("Valid".to_string(), serde_json::Value::Bool(true));
        file_contents = serde_json::to_string(&key_values)?;
        let file_contents_raw = file_contents.as_bytes();
        file_io.set_len(file_contents_raw.len() as u64)?;
        file_io.write_all(file_contents_raw)?;
        file_io.flush()?;

        Ok(())
    }

    fn update_inventory(&self) -> Result<(), Box<dyn Error>> {
        let mut file_io = OpenOptions::new().write(true).open(self.inventory_path.clone())?;

        let file_contents_raw = DEFAULT_INVENTORY.as_bytes();
        file_io.set_len(file_contents_raw.len() as u64)?;
        file_io.write_all(file_contents_raw)?;
        file_io.flush()?;

        Ok(())
    }

}

#[derive(Clone, Data, PartialEq)]
enum MainView {
    Error,
    Data,
}

#[derive(Clone, Data, Lens)]
struct UiState {
    #[data(eq)]
    #[lens(name = "profile_file_lens")]
    profile_file: PathBuf,
    #[lens(name = "profile_lens")]
    profile: Profile,
    #[data(eq)]
    #[lens(name = "characters_file_lens")]
    characters_file: PathBuf,
    #[lens(name = "characters_lens")]
    characters: Vector<Character>,
    #[lens(name = "error_lens")]
    error: Option<String>,
}

impl UiState {
    pub fn new() -> Result<UiState, Box<dyn Error>> {
        let dirs = BaseDirs::new().ok_or::<Box<dyn Error>>("Unable to find %APPDATA%\\Local\\".into())?;
        let data_local_dir = dirs.data_local_dir().join("Icarus").join("Saved").join("Offline");
        let profile_file = data_local_dir.join("Profile.json");
        let characters_file = data_local_dir.join("Characters.json");

        if !profile_file.exists() || !characters_file.exists() {
            Err(format!(
                "One or both of [{}] and [{}] do not exist - please open Icarus and create an Offline character before running this tool",
                profile_file.to_string_lossy(),
                characters_file.to_string_lossy()
            ))?
        }

        let mut profile_file_io = OpenOptions::new().write(true).read(true).open(profile_file.clone())?;
        let mut profile_string = String::new();
        profile_file_io.read_to_string(&mut profile_string)?;
        let profile: Profile = serde_json::from_str(&profile_string)?;

        let mut character_string = String::new();
        let mut character_file_io = OpenOptions::new().write(true).read(true).open(characters_file.clone())?;
        character_file_io.read_to_string(&mut character_string)?;

        let chars: Characters = serde_json::from_str(&character_string)?;
        let mut characters = Vec::<Character>::with_capacity(chars.characters_json.len());
        for c in chars.characters_json {
            let mut character: Character = serde_json::from_str(&c)?;
            character.inventory_path = data_local_dir.join("Inventory").join(format!("InventoryID_{}.json", character.character_slot as i8));
            character.loadout_path = data_local_dir.join("Loadout").join(format!("Slot_{}.json", character.character_slot as i8));
            characters.push(character);
        }
        characters.sort_by(|a, b|{
            if let Some(c) = a.character_slot.partial_cmp(&b.character_slot) {
                c
            } else {
                panic!("Could not compare floating points")
            }
        });
        let data = UiState {
            profile_file,
            profile,
            characters_file,
            characters: Vector::from(characters),
            error: None,
        };

        Ok(data)
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let mut profile_file_io = OpenOptions::new().write(true).open(self.profile_file.clone())?;
        let profile_string = serde_json::to_string(&self.profile)?;
        let profile_bytes = profile_string.as_bytes();
        profile_file_io.set_len(profile_bytes.len() as u64)?;
        profile_file_io.write_all(profile_bytes)?;
        profile_file_io.flush()?;

        let mut character_file_io = OpenOptions::new().write(true).open(self.characters_file.clone())?;
        let mut characters = Characters {
            characters_json: Vector::new(),
        };

        for c in &self.characters {
            let character_string = serde_json::to_string(c)?;
            characters.characters_json.push_front(character_string);
        }

        let characters_string = serde_json::to_string(&characters)?;
        let characters_bytes = characters_string.as_bytes();

        character_file_io.set_len(characters_bytes.len() as u64)?;
        character_file_io.write_all(characters_bytes)?;
        character_file_io.flush()?;

        Ok(())
    }
}

#[derive(Clone, Data)]
struct CharTabs {

}

impl TabsPolicy for CharTabs {
    type Key = usize;
    type Input = UiState;
    type BodyWidget = Flex<UiState>;
    type LabelWidget = Label<UiState>;
    type Build = ();

    fn tabs_changed(&self, old_data: &Self::Input, data: &Self::Input) -> bool {
        !old_data.characters.eq(&data.characters)
    }

    fn tabs(&self, data: &Self::Input) -> Vec<Self::Key> {
        data.characters.iter().map(|x| x.character_slot as usize).collect()
    }

    fn tab_info(&self, key: Self::Key, _: &Self::Input) -> TabInfo<Self::Input> {
        println!("Loading tab info for key {}", key);
        TabInfo::new(
            LabelText::from(move |state: &UiState, _: &Env|{
                state.characters.iter().find(|x| x.character_slot as usize == key).map(|x| x.character_name.clone()).expect("unreachable")
            }),
            false,
        )
    }

    fn tab_body(&self, key: Self::Key, data: &Self::Input) -> Self::BodyWidget {
        println!("Loading tab body for key {}", key);
        let idx = data.characters.index_of(data.characters.iter().find(|x| x.character_slot as usize == key).expect("not possible")).expect("not possible");
        println!("Found idx {}", idx);
        let character_lens = UiState::characters_lens.index(idx);
        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(Flex::row()
                .with_child(Label::new(format!("Current Prospect: {}", data.characters[idx].location)))
            ).with_default_spacer()
            .with_child(Flex::row()
                .with_child(Label::new("XP"))
                .with_default_spacer()
                .with_child(ValueTextBox::new(TextBox::new(), ParseFormatter::<f64>::new()).lens(character_lens.clone().then(Character::xp)))
                .with_default_spacer()
                .with_child(Button::new("Max Level").on_click(|_, state: &mut Character, _| state.level_to_max() ).lens(character_lens.clone()))
            ).with_default_spacer()
            .with_child(Flex::row()
                .with_child(Label::new("XP Debt"))
                .with_default_spacer()
                .with_child(ValueTextBox::new(TextBox::new(), ParseFormatter::<f64>::new()).lens(character_lens.clone().then(Character::xp_debt)))
            ).with_default_spacer()
            .with_child(Flex::row()
                .with_child(Label::new("Dead"))
                .with_default_spacer()
                .with_child(Checkbox::new("").lens(character_lens.clone().then(Character::is_dead)).disabled_if(|_, _| true))
            ).with_default_spacer()
            .with_child(Flex::row()
                .with_child(Label::new("Abandoned"))
                .with_default_spacer()
                .with_child(Checkbox::new("")
                    .disabled_if(|state: &bool, _ctx| !*state)
                    .lens(character_lens.clone().then(Character::is_abandoned)))
                .with_child(Button::new("Restore Character")
                    .on_click(|_ctx, t: &mut Character, _env|{ t.restore().expect("Restoring character failed unexpectedly") })
                    .disabled_if(|state: &Character, _ctx| !state.is_abandoned)
                    .lens(character_lens.clone()))
            ).with_default_spacer()
            .with_child(Flex::row()
                .with_child(Button::new("Reset Talents").on_click(|_ctx, t: &mut Character, _env| t.reset_talents()).lens(character_lens.clone()))
            ).with_default_spacer()
            .with_child(Flex::row()
                .with_child(Button::new("Reset Blueprints").on_click(|_ctx, t: &mut Character, _env| t.reset_blueprints()).lens(character_lens.clone()))
            ).with_default_spacer()
            .with_child(Flex::row()
                .with_child(Button::new("Unlock All Talents").on_click(|_ctx, t: &mut Character, _env| t.unlock_all_talents()).lens(character_lens.clone()))
            ).with_default_spacer()
            .with_child(Flex::row()
                .with_child(Button::new("Unlock All Blueprints").on_click(|_ctx, t: &mut Character, _env| t.unlock_all_blueprints()).lens(character_lens.clone()))
            ).with_default_spacer()
            .with_child(Flex::row()
                .with_child(Checkbox::new("Exotic Mining Unlocked").lens(character_lens.clone().then(Character::unlocked_flags).then(FlagLens{ flag: EXOTIC_MINING_FLAG })))
            ).with_default_spacer()
            .with_child(Flex::row()
                .with_child(Checkbox::new("Exotic Extraction Unlocked").lens(character_lens.clone().then(Character::unlocked_flags).then(FlagLens{ flag: EXOTIC_EXTRACTION_FLAG })))
            ).with_default_spacer()
            .with_child(Flex::row()
                .with_child(Button::new("Save").on_click(|_ctx, t: &mut UiState, _env| t.save().expect("Error saving profile and/or character data") ))
            )
    }

    fn tab_label(&self, _: Self::Key, info: TabInfo<Self::Input>, _: &Self::Input) -> Self::LabelWidget {
        Self::default_make_label(info)
    }
}

fn ui_builder() -> impl Widget<UiState> {
    let view_switcher = ViewSwitcher::new(
        |data: &UiState, _env| { if data.error.is_some() { MainView::Error } else { MainView::Data }},
        |selector, data: &UiState, _env| {
            Box::new(match selector {
                MainView::Data => {
                    let label_credits = Label::<UiState>::new("Credits: ");
                    let textbox_credits = ValueTextBox::new(TextBox::new(), ParseFormatter::<f64>::new())
                        .fix_width(100.0)
                        .lens(UiState::profile_lens.then(Profile::meta_resources).then(Credit()).then(MetaResources::count));
                    let label_exotics = Label::<UiState>::new("Exotics: ");
                    let textbox_exotics = ValueTextBox::new(TextBox::new(), ParseFormatter::<f64>::new())
                        .fix_width(100.0)
                        .lens(UiState::profile_lens.then(Profile::meta_resources).then(Exotic()).then(MetaResources::count));
                    let tabs = Tabs::for_policy(CharTabs{})/*.lens(UiState)*/;
                    let layout = Flex::column()
                        .with_child(Flex::row().with_child(label_credits).with_default_spacer().with_child(textbox_credits))
                        .with_default_spacer()
                        .with_child(Flex::row().with_child(label_exotics).with_default_spacer().with_child(textbox_exotics))
                        .with_default_spacer()
                        .with_child(Flex::row()
                            .with_child(Button::new("Unlock All Prospects").on_click(|_ctx, t: &mut Profile, _env| t.unlock_all_prospects()).lens(UiState::profile_lens))
                        )
                        .with_default_spacer()
                        .with_child(Flex::row()
                            .with_child(Button::new("Unlock All Workshop Items").on_click(|_ctx, t: &mut Profile, _env| t.unlock_all_workshop_items()).lens(UiState::profile_lens))
                        )
                        .with_default_spacer()
                        .with_child(Flex::row()
                            .with_child(Button::new("Save").on_click(|_ctx, t: &mut UiState, _env| t.save().expect("Error saving data")))
                        )
                        .with_default_spacer()
                        .with_flex_child(tabs, 1.0);
                    Align::centered(layout)
                },
                MainView::Error => Align::centered(Label::new(format!("Error occurred during startup: {}", data.error.as_ref().unwrap_or(&"Unknown Error".to_string())))),
            })
        }
    );

    view_switcher
}

fn main() -> Result<(), Box<dyn Error>> {
    let main_window = WindowDesc::new(ui_builder()).title("Icarus Offline Character Editor").window_size((440.0, 600.0));
    let data = UiState::new();
    match data {
        Ok(d) => AppLauncher::with_window(main_window)
            .log_to_console()
            .launch(d)?,
        Err(e) => AppLauncher::with_window(main_window)
            .log_to_console()
            .launch(UiState {
                profile_file: Default::default(),
                profile: Profile {
                    user_id: "".to_string(),
                    meta_resources: Default::default(),
                    unlocked_flags: Default::default(),
                    talents: Default::default()
                },
                characters_file: Default::default(),
                characters: Default::default(),
                error: Some(format!("Error: {}", e)),
            })?,
    }

    Ok(())
}