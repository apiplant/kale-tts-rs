(() => {
  // ../frontend/dizione/tables.js
  var ABBREVIATIONS = { "mr": ["title", "mister"], "mrs": ["title", "mrs"], "ms": ["title", "ms"], "prof": ["title", "professor"], "gen": ["title", "general"], "col": ["title", "colonel"], "lt": ["title", "lieutenant"], "capt": ["title", "captain"], "sgt": ["title", "sergeant"], "cpl": ["title", "corporal"], "rev": ["title", "reverend"], "hon": ["title", "honorable"], "gov": ["title", "governor"], "sen": ["title", "senator"], "rep": ["title", "representative"], "pres": ["title", "president"], "mt": ["title", "mount"], "ft": ["title", "fort"], "fr": ["title", "father"], "jr": ["always", "junior"], "sr": ["always", "senior"], "vs": ["always", "versus"], "etc": ["always", "et cetera"], "inc": ["always", "incorporated"], "ltd": ["always", "limited"], "co": ["always", "company"], "corp": ["always", "corporation"], "ave": ["always", "avenue"], "blvd": ["always", "boulevard"], "rd": ["always", "road"], "ln": ["always", "lane"], "hwy": ["always", "highway"], "apt": ["always", "apartment"], "bldg": ["always", "building"], "dept": ["always", "department"], "govt": ["always", "government"], "univ": ["always", "university"], "assn": ["always", "association"], "bros": ["always", "brothers"], "approx": ["always", "approximately"], "est": ["always", "established"], "misc": ["always", "miscellaneous"], "tel": ["always", "telephone"], "max": ["always", "maximum"], "min": ["always", "minimum"], "sq": ["always", "square"], "pkwy": ["always", "parkway"], "appt": ["always", "appointment"], "qty": ["always", "quantity"], "acct": ["always", "account"], "avg": ["always", "average"], "incl": ["always", "including"], "excl": ["always", "excluding"], "attn": ["always", "attention"], "fl": ["small", "floor"], "ct": ["small", "court"], "no": ["number", "number"], "nos": ["number", "numbers"], "p": ["number", "page"], "pp": ["number", "pages"], "fig": ["number", "figure"], "vol": ["number", "volume"], "ch": ["number", "chapter"], "sec": ["number", "section"], "ref": ["number", "reference"], "ext": ["number", "extension"], "ste": ["number", "suite"], "rm": ["number", "room"], "ca": ["number", "circa"], "jan": ["always", "January"], "feb": ["always", "February"], "mar": ["always", "March"], "apr": ["always", "April"], "jun": ["always", "June"], "jul": ["always", "July"], "aug": ["always", "August"], "sep": ["always", "September"], "sept": ["always", "September"], "oct": ["always", "October"], "nov": ["always", "November"], "dec": ["always", "December"], "mon": ["always", "Monday"], "tue": ["always", "Tuesday"], "tues": ["always", "Tuesday"], "wed": ["always", "Wednesday"], "thu": ["always", "Thursday"], "thur": ["always", "Thursday"], "thurs": ["always", "Thursday"], "fri": ["always", "Friday"] };
  var UNITS = { "km": ["kilometer", "kilometers"], "kg": ["kilogram", "kilograms"], "cm": ["centimeter", "centimeters"], "mm": ["millimeter", "millimeters"], "ml": ["milliliter", "milliliters"], "mg": ["milligram", "milligrams"], "kb": ["kilobyte", "kilobytes"], "mb": ["megabyte", "megabytes"], "gb": ["gigabyte", "gigabytes"], "tb": ["terabyte", "terabytes"], "hz": ["hertz", "hertz"], "khz": ["kilohertz", "kilohertz"], "mhz": ["megahertz", "megahertz"], "ghz": ["gigahertz", "gigahertz"], "kw": ["kilowatt", "kilowatts"], "kwh": ["kilowatt hour", "kilowatt hours"], "lb": ["pound", "pounds"], "lbs": ["pound", "pounds"], "oz": ["ounce", "ounces"], "ft": ["foot", "feet"], "mi": ["mile", "miles"], "min": ["minute", "minutes"], "mins": ["minute", "minutes"], "hr": ["hour", "hours"], "hrs": ["hour", "hours"], "sec": ["second", "seconds"], "secs": ["second", "seconds"], "ms": ["millisecond", "milliseconds"], "mph": ["mile per hour", "miles per hour"], "kph": ["kilometer per hour", "kilometers per hour"], "km/h": ["kilometer per hour", "kilometers per hour"], "\xB0c": ["degree Celsius", "degrees Celsius"], "\xB0f": ["degree Fahrenheit", "degrees Fahrenheit"], "\xB0": ["degree", "degrees"], "m/s": ["meter per second", "meters per second"], "yd": ["yard", "yards"], "nm": ["nanometer", "nanometers"], "\u03BCm": ["micrometer", "micrometers"], "\u03BCg": ["microgram", "micrograms"], "mcg": ["microgram", "micrograms"], "mah": ["milliamp hour", "milliamp hours"], "hp": ["horsepower", "horsepower"], "tsp": ["teaspoon", "teaspoons"], "tbsp": ["tablespoon", "tablespoons"], "db": ["decibel", "decibels"], "kcal": ["kilocalorie", "kilocalories"], "px": ["pixel", "pixels"], "rpm": ["revolution per minute", "revolutions per minute"], "bpm": ["beat per minute", "beats per minute"], "fps": ["frame per second", "frames per second"], "mpg": ["mile per gallon", "miles per gallon"], "psi": ["pound per square inch", "pounds per square inch"], "kbps": ["kilobit per second", "kilobits per second"], "mbps": ["megabit per second", "megabits per second"], "gbps": ["gigabit per second", "gigabits per second"], "yr": ["year", "years"], "yrs": ["year", "years"], "wk": ["week", "weeks"], "wks": ["week", "weeks"] };
  var EXACT_UNITS = { "m": ["meter", "meters"], "g": ["gram", "grams"], "l": ["liter", "liters"], "L": ["liter", "liters"], "h": ["hour", "hours"], "V": ["volt", "volts"], "W": ["watt", "watts"], "MW": ["megawatt", "megawatts"], "p": ["penny", "pence"], "ha": ["hectare", "hectares"] };
  var CHAT = { "lol": "*LOL", "lmao": "*LMAO", "rofl": "*ROFL", "omg": "*OMG", "omfg": "*OMG", "wtf": "*WTF", "wth": "what the heck", "idk": "I don't know", "idc": "I don't care", "btw": "by the way", "imo": "in my opinion", "imho": "in my humble opinion", "tbh": "to be honest", "brb": "be right back", "bbl": "be back later", "ttyl": "talk to you later", "gtg": "got to go", "g2g": "got to go", "afaik": "as far as I know", "iirc": "if I recall correctly", "fyi": "*FYI", "faq": "*FAQ", "diy": "*DIY", "aka": "*AKA", "eta": "*ETA", "tba": "*TBA", "tbd": "*TBD", "rsvp": "*RSVP", "asap": "*ASAP", "smh": "*SMH", "irl": "in real life", "dm": "*DM", "dms": "*DM s", "thx": "thanks", "tnx": "thanks", "ty": "thank you", "tysm": "thank you so much", "yw": "you're welcome", "np": "no problem", "pls": "please", "plz": "please", "ppl": "people", "msg": "message", "msgs": "messages", "pic": "pic", "pics": "pics", "u": "you", "ur": "your", "r": "are", "k": "okay", "kk": "okay", "ok": "okay", "nvm": "never mind", "bc": "because", "cuz": "because", "coz": "because", "tho": "though", "thru": "through", "w/": "with", "w/o": "without", "b4": "before", "gr8": "great", "l8r": "later", "2day": "today", "2moro": "tomorrow", "2nite": "tonight", "rn": "right now", "jk": "just kidding", "ikr": "I know right", "omw": "on my way", "hbu": "how about you", "wyd": "what are you doing", "fr": "for real", "ngl": "not gonna lie", "icl": "I can't lie", "istg": "I swear to god", "idek": "I don't even know", "tbf": "to be fair", "iykyk": "if you know you know", "wsg": "what's good", "gng": "gang", "pov": "*POV", "fyp": "*FYP", "afk": "*AFK", "npc": "*NPC", "gg": "*GG", "haha": "ha ha", "hahaha": "ha ha ha", "hahahaha": "ha ha ha", "hehe": "heh heh", "xd": "-", "xoxo": "-", "n/a": "not applicable", "ctrl": "control", "cmd": "command", "esc": "escape" };
  var ENDINGS = { "com": "word", "org": "word", "net": "word", "gov": "word", "info": "word", "biz": "word", "app": "word", "dev": "word", "co": "word", "edu": "letters", "io": "letters", "ai": "letters", "sh": "letters", "ly": "letters", "tv": "letters", "fm": "letters", "gg": "letters", "xyz": "letters", "uk": "letters", "eu": "letters", "de": "letters", "fr": "letters", "es": "letters", "nl": "letters", "se": "letters", "ch": "letters", "ca": "letters", "au": "letters", "jp": "letters", "cn": "letters", "ru": "letters", "br": "letters", "pdf": "letters", "txt": "letters", "csv": "letters", "tsv": "letters", "doc": "word", "docx": "letters", "xls": "letters", "xlsx": "letters", "ppt": "letters", "pptx": "letters", "odt": "letters", "rtf": "letters", "tex": "letters", "md": "letters", "epub": "word", "json": "word", "yaml": "word", "yml": "letters", "toml": "word", "xml": "letters", "html": "letters", "htm": "letters", "sql": "letters", "log": "word", "conf": "word", "cfg": "letters", "ini": "letters", "env": "letters", "lock": "word", "png": "letters", "jpg": "letters", "jpeg": "word", "gif": "word", "svg": "letters", "bmp": "letters", "tiff": "word", "psd": "letters", "webp": "letters", "heic": "letters", "mp3": "letters", "mp4": "letters", "m4a": "letters", "wav": "word", "flac": "word", "midi": "word", "mpeg": "word", "mov": "letters", "avi": "letters", "mkv": "letters", "webm": "letters", "exe": "letters", "dll": "letters", "apk": "letters", "dmg": "letters", "iso": "letters", "zip": "word", "tar": "word", "gz": "letters", "tgz": "letters", "rar": "letters", "jar": "letters", "bin": "word", "bat": "word", "bak": "letters", "tmp": "letters", "py": "letters", "js": "letters", "ts": "letters", "jsx": "letters", "tsx": "letters", "css": "letters", "rb": "letters", "rs": "letters", "cpp": "letters", "php": "letters", "java": "word", "swift": "word", "kt": "letters", "ipynb": "letters" };
  var HOMOGRAPH_RULES = [["read", "past", "aux: have has had having haven't hasn't hadn't was were been being is are be am got"], ["read", "present", "aux: to will would can could should shall may might must do does did don't doesn't didn't let let's please cannot can't won't wouldn't couldn't shouldn't"], ["read", "present", "first"], ["live", "vrb", "before: i you we they who people to"], ["live", "adj", "next: music show shows broadcast performance performances concert concerts stream streaming coverage tv television album recording audience action event events demo feed wire ammunition"], ["wound", "vrb", "aux: has have had having was were been got get tightly"], ["wound", "vrb", "next: up around round down through back tightly"], ["wound", "vrb", "before: he she it and then"], ["bow", "nou-ship", "aux: take took takes taking taken to will would must"], ["bow", "nou-knot", "near: tie tied tying ties ribbon ribbons hair arrow arrows violin cello fiddle string strings rainbow gift shoelace shoelaces"], ["bow", "nou-ship", "near: ship ships boat deck stern captain audience curtain applause stage head knee knees waist"], ["bass", "corp", "near: fish fishing fished caught catch lake river pond sea striped largemouth smallmouth angler trout"], ["lead", "nou", "near: pipe pipes paint pencil poisoning metal metals bullet bullets zinc copper tin mercury ore"], ["minute", "adj", "before: is are was were so very extremely too such ; next: </s> , . ! ? and but details detail amount amounts quantities quantity particles differences changes fraction"], ["content", "adj-nou-vrb", "before: is are was were am be been feel feels felt seem seems seemed quite so very perfectly not ; next: </s> , . ! ? with to in and"]];

  // ../frontend/dizione/spoken.js
  var SPELL = "\uE000";
  var END = "\uE001";
  var ONES = "zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen".split(" ");
  var TENS = "twenty thirty forty fifty sixty seventy eighty ninety".split(" ");
  var SCALES = ["thousand", "million", "billion", "trillion", "quadrillion"];
  var ORDINAL = { one: "first", two: "second", three: "third", five: "fifth", eight: "eighth", nine: "ninth", twelve: "twelfth" };
  var MONTHS = "January February March April May June July August September October November December".split(" ");
  var MONTH_NUMBER = Object.fromEntries(MONTHS.map((name, index) => [name.toLowerCase().slice(0, 3), index + 1]));
  var MONTH_PATTERN = MONTHS.join("|") + "|Jan|Feb|Mar|Apr|Jun|Jul|Aug|Sept|Sep|Oct|Nov|Dec";
  var DAY_NAMES = { mon: "Monday", tue: "Tuesday", wed: "Wednesday", thu: "Thursday", fri: "Friday", sat: "Saturday", sun: "Sunday" };
  var DAY_PATTERN = "Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday|Thurs|Thur|Tues|Mon|Tue|Wed|Thu|Fri|Sat|Sun";
  var CURRENCY = {
    "$": ["dollar", "dollars", "cent", "cents"],
    "\xA3": ["pound", "pounds", "penny", "pence"],
    "\u20AC": ["euro", "euros", "cent", "cents"],
    "\xA5": ["yen", "yen", "", ""],
    "\u20B9": ["rupee", "rupees", "paisa", "paise"]
  };
  Object.assign(CURRENCY, {
    USD: CURRENCY["$"],
    EUR: CURRENCY["\u20AC"],
    GBP: CURRENCY["\xA3"],
    JPY: CURRENCY["\xA5"],
    INR: CURRENCY["\u20B9"],
    CHF: ["Swiss franc", "Swiss francs", "centime", "centimes"],
    CAD: ["Canadian dollar", "Canadian dollars", "cent", "cents"],
    AUD: ["Australian dollar", "Australian dollars", "cent", "cents"],
    CNY: ["yuan", "yuan", "", ""]
  });
  var CURRENCY_CODES = "USD|EUR|GBP|JPY|INR|CHF|CAD|AUD|CNY";
  var SCALE_LETTER = { k: "thousand", m: "million", b: "billion", bn: "billion", t: "trillion" };
  var FRACTION = { 2: ["half", "halves"], 4: ["quarter", "quarters"] };
  var SYMBOL_WORDS = {
    "&": "and",
    "@": "at",
    "+": "plus",
    "=": "equals",
    "%": "percent",
    "\xD7": "times",
    "\xF7": "divided by",
    "\xB1": "plus or minus",
    "\xA9": "copyright",
    "\xA7": "section",
    "\xB0": "degrees",
    "\u2265": "greater than or equal to",
    "\u2264": "less than or equal to",
    "\u2260": "not equal to",
    "\u2248": "approximately",
    "\u221E": "infinity",
    "\u221A": "square root of",
    "\u03C0": "pi"
  };
  var COMPARISONS = { ">=": "greater than or equal to", "<=": "less than or equal to", "!=": "not equal to" };
  var words = (text) => new Set(text.split(" "));
  var SENTENCE_STARTERS = words("the he she it i we they this that there but and in on at a an if when what why how who my our you then so now after before however");
  var YEAR_CONTEXT = words("in since by from until till before after during around circa of year early late mid summer winter spring fall autumn class");
  var PHONE_WORDS = words("call dial ring phone telephone tel mobile cell fax hotline text sms whatsapp");
  var PHONE_LINKS = words("to on at or us me him her them back");
  var PIN_WORDS = words("code pin extension ext passcode password zip postcode cvv cvc otp");
  var STRONG_WORDS = words("number id reference ref serial account tracking booking confirmation");
  var WEAK_WORDS = words("order ticket invoice case claim policy receipt transaction parcel shipment container payment reservation membership customer batch lot item job issue bug request");
  var LINK_WORDS = words("is was are were ends ending in with starts starting begins beginning");
  var NOT_COUNTED = words("is was has does goes ships arrives departs leaves shows needs blocks remains includes contains says reads starts ends closes opens runs takes comes this his its us as yes plus minus less");
  var NUMBERED = words("ref reference serial account order invoice tracking part model phone tel fax id room flight case ticket policy licence license booking customer vat tax reg registration");
  var DATE_BEFORE = words("on dated due until till by since from before after for shipped delivered arriving arrives arrive arrived expires expired ends ending starts starting begins beginning monday tuesday wednesday thursday friday saturday sunday mon tue tues wed thu thur thurs fri sat sun");
  var CLASSIC_FRACTIONS = /* @__PURE__ */ new Set([2, 3, 4, 5, 6, 8, 10, 16, 32, 64, 100]);
  var RATIO_WORDS = words("ratio ratios scale odds aspect mix split proportion");
  var DURATION_WORDS = words("lasted took duration runtime recording video clip track lap pace length long");
  var CLOCK_WORDS = words("window slot open opens opening hours shift session served service available booked appointment delivery arrive arrives arriving");
  var CAPITAL_WORDS = words("IN ON AT TO OF BY OR IS AS IF NO SO WE HE MY ME DO GO UP AN AM PM BE IT US THE AND FOR BUT NOT ALL");
  var SHORT_WORDS = words("my to in on of go up no do me we us it is at by be so or an as if he hi ok");
  var ZONES = "ET|CT|MT|PT|EST|EDT|CST|CDT|MST|MDT|PST|PDT|GMT|UTC|BST|CET|CEST|IST|AEST|AEDT|JST";
  var SIZES = "XXXL|XXL|XXS|XL|XS|S|M|L";
  var LENGTHS = "mm|cm|km|m|ft|in|inch|inches|feet|foot|metres?|meters?|yards?|yd|px|pixels?";
  var VERSIONED = "version|ver|release|firmware|patch|kernel|build|node|nodejs|ios|ipados|macos|watchos|tvos|android|chrome|firefox|safari|java|php|ruby|rust|swift|kotlin|golang|django|rails|react|angular|vue|docker|kubernetes|ubuntu|debian|fedora|windows|linux|postgres|postgresql|mysql|redis|numpy|pytorch|tensorflow|cuda|xcode|gcc|clang|unity|minecraft|typescript|bluetooth|usb|hdmi|opengl|vulkan|directx|upgrade[ds]?|update[ds]?|downgrade[ds]?";
  var SEPARATORS = { ".": "dot", "/": "slash", "@": "at", "-": "dash", "_": "underscore", ":": "colon", "=": "equals", "&": "and", "+": "plus" };
  var HALF_DAY = String.raw`(?:\s?([AaPp])\.[Mm]\.?|\s?([AaPp])[Mm]\b)`;
  var AMOUNT = String.raw`(?<whole>\d{1,3}(?:,\d{3})+|\d+)(?:\.(?<fraction>\d+))?`;
  var MONEY = new RegExp(String.raw`(?:(?<![\w.])(?<minus>-)\s?)?(?<symbol>[$£€¥₹]|(?<![A-Za-z])(?:${CURRENCY_CODES})(?![A-Za-z]))\s?` + String.raw`(?<inner>-)?${AMOUNT}(?:\s?(?<scale>thousand|million|billion|trillion|bn|[kKmMbB])(?![A-Za-z]))?`, "g");
  var MONEY_AFTER = new RegExp(String.raw`(?<![\w.,])${AMOUNT}\s?(?<symbol>${CURRENCY_CODES})(?![A-Za-z])`, "g");
  var VULGAR = /(?<![\d.,/])(?:(\d+)\s?)?([½⅓⅔¼¾⅕⅖⅗⅘⅙⅚⅐⅛⅜⅝⅞⅑⅒])/g;
  var ADDRESS = new RegExp(
    String.raw`(?<url>(?:https?:\/\/|www\.)[^\s<>"']+)` + String.raw`|(?<mail>[\w.+-]+@[\w-]+(?:\.[\w-]+)+)` + String.raw`|(?<file>(?<![\w@./-])(?<folder>\/?(?:[\w~-]+\/)+)?(?<stem>[\w-]+(?:\.[\w-]+)*)\.(?<ending>[A-Za-z][A-Za-z0-9]{0,9})` + String.raw`(?![\w-])(?<path>\/[^\s<>"']*)?)`,
    "g"
  );
  var DAY_RANGE = new RegExp(String.raw`\b(${DAY_PATTERN}|${MONTH_PATTERN})\b(\.?)\s?[-–]\s?(${DAY_PATTERN}|${MONTH_PATTERN})\b(\.?)`, "g");
  var RATIO = /(?<![\d:.,])(\d{1,3}):(\d{1,3}(?:,\d{3})+|\d+)(?![\d:]|[.,]\d)/g;
  var CODE = /(?<![\w.,/-])(#)?([A-Za-z0-9]+(?:-[A-Za-z0-9]+)+)(?![\w-]|[.,]\d)/g;
  var SLASHED = /(?<![\w.,/-])(\d+(?:\/\d+){2,})(?![\w/]|[.,]\d)/g;
  var GROUP_CORE = String.raw`[A-Z0-9]*\d[A-Z0-9]*(?: [A-Z0-9]*\d[A-Z0-9]*)+`;
  var GROUPED = new RegExp(String.raw`(?<![\w.,:/#+-])(\+)?(?:([A-Z]{2}) (${GROUP_CORE})(?: ([A-Z]{2})(?!\w))?|(${GROUP_CORE}))` + String.raw`(?!\w|[.,:/-]\d|\s?[%°])`, "g");
  var GLUED = /(?<![\w'.])(?=[A-Za-z0-9]*[A-Za-z])(?=[A-Za-z0-9]*\d)[A-Za-z0-9]+(?![\w']|\.\d)/g;
  var EMOJI = /[\u{1F000}-\u{1FAFF}☀-➿\u{1F1E6}-\u{1F1FF}←-⇿⬀-⯿️‍⃣]+/gu;
  var EMOTICON = new RegExp(
    String.raw`(?<![A-Za-z0-9])(?:[:;=]['^-]?[DPpOo3|/\\*\]\[}{><]+|[)(\]\[][-^']?[:;=]|<3+|<\/3|\^_?\^|-_-|[oO]_[oO]|>_<|T_T|\\o\/|:'[()])(?![A-Za-z0-9])` + String.raw`|[:;=]['^-]?[)(]+`,
    "g"
  );
  var isUpper = (s) => s !== s.toLowerCase() && s === s.toUpperCase();
  var isAlpha = (s) => /^\p{L}+$/u.test(s);
  var isDigits = (s) => /^\d+$/.test(s);
  var int = (s) => Number.parseInt(s, 10);
  function cardinal(number) {
    if (number < 0) return "minus " + cardinal(-number);
    if (number < 20) return ONES[number];
    if (number < 100) {
      const tens = Math.floor(number / 10), ones = number % 10;
      return TENS[tens - 2] + (ones ? " " + ONES[ones] : "");
    }
    if (number < 1e3) {
      const rest = number % 100;
      return ONES[Math.floor(number / 100)] + " hundred" + (rest ? " " + cardinal(rest) : "");
    }
    if (number >= 1e3 ** (SCALES.length + 1)) return digits(String(number));
    const parts = [];
    for (let power = SCALES.length; power > 0; power--) {
      const unit = 1e3 ** power, count = Math.floor(number / unit);
      number %= unit;
      if (count) parts.push(cardinal(count) + " " + SCALES[power - 1]);
    }
    if (number) parts.push(cardinal(number));
    return parts.join(" ");
  }
  function ordinal(number) {
    const list = cardinal(number).split(" ");
    const last = list[list.length - 1];
    list[list.length - 1] = ORDINAL[last] ?? (last.endsWith("y") ? last.slice(0, -1) + "ieth" : last + "th");
    return list.join(" ");
  }
  var digits = (text, zero = "zero") => [...text].filter((c) => /\d/.test(c)).map((c) => c === "0" ? zero : ONES[+c]).join(" ");
  function year(number) {
    if (!(number >= 1e3 && number <= 2999)) return cardinal(number);
    const high = Math.floor(number / 100), low = number % 100;
    if (low === 0) return high % 10 === 0 ? cardinal(number) : cardinal(high) + " hundred";
    if (high === 20 && low < 10) return "two thousand " + ONES[low];
    return cardinal(high) + (low < 10 ? " oh " + ONES[low] : " " + cardinal(low));
  }
  function pairs(number) {
    const high = Math.floor(number / 100), low = number % 100;
    if (!(number >= 100 && number <= 9999) || low === 0 && (high < 10 || high % 10 === 0)) return cardinal(number);
    if (low === 0) return cardinal(high) + " hundred";
    return cardinal(high) + (low < 10 ? " oh " + ONES[low] : " " + cardinal(low));
  }
  function numberWords(text) {
    const plain = text.replace(/,/g, "");
    const dot2 = plain.indexOf(".");
    const whole = dot2 < 0 ? plain : plain.slice(0, dot2), fraction = dot2 < 0 ? "" : plain.slice(dot2 + 1);
    if (fraction) return (whole ? cardinal(int(whole)) : "") + " point " + digits(fraction);
    if (whole.length > 1 && whole.startsWith("0") || whole.length > 15) return digits(whole);
    return cardinal(int(whole));
  }
  function duration(hours, minutes, seconds) {
    const parts = [[hours, "hour"], [minutes, "minute"], [seconds, "second"]].filter(([count]) => count).map(([count, name]) => `${cardinal(count)} ${count === 1 ? name : name + "s"}`);
    if (parts.length < 2) return parts[0] ?? "zero seconds";
    return parts.slice(0, -1).join(", ") + " and " + parts[parts.length - 1];
  }
  function pluralOf(spoken) {
    const at = spoken.lastIndexOf(" ");
    const head = at < 0 ? "" : spoken.slice(0, at), last = spoken.slice(at + 1);
    return (head ? head + " " : "") + (last.endsWith("y") ? last.slice(0, -1) + "ies" : last + "s");
  }
  function fractionWords(top, bottom, article = false) {
    const [one, many] = FRACTION[bottom] ?? [ordinal(bottom), ordinal(bottom) + "s"];
    if (top !== 1) return `${cardinal(top)} ${many}`;
    return (article ? "aeiou".includes(one[0]) ? "an " : "a " : "one ") + one;
  }
  var versionPart = (part) => part.length > 2 ? digits(part) : numberWords(part);
  var minor = (part) => part.startsWith("0") ? digits(part, "oh") : cardinal(int(part));
  var shortNumber = (text) => !text.startsWith("0") && (text.length <= 2 || /^0+$/.test(text.slice(1)));
  var spelled = (letters) => `${SPELL}${letters}${END}`;
  function before(text, position, count = 4) {
    const parts = text.slice(Math.max(0, position - 80), position).split(/[.!?;…]\s|[()"]/);
    return (parts[parts.length - 1].match(/[A-Za-z]+/g) ?? []).map((word) => word.toLowerCase()).slice(-count);
  }
  function identifier(text, position) {
    const list = before(text, position);
    if (list.length === 0) return null;
    const last = list[list.length - 1];
    if (PHONE_WORDS.has(last) || PHONE_LINKS.has(last) && list.slice(-3, -1).some((word) => PHONE_WORDS.has(word))) return "phone";
    let index = list.length - 1;
    while (index > 0 && LINK_WORDS.has(list[index])) index--;
    for (const [kind, group] of [["phone", PHONE_WORDS], ["pin", PIN_WORDS], ["strong", STRONG_WORDS], ["weak", WEAK_WORDS]]) {
      if (group.has(list[index])) return kind;
    }
    return last === "enter" ? "enter" : null;
  }
  function addressDigits(text) {
    if (text.length <= 2 && !(text.length === 2 && text[0] === "0")) return cardinal(int(text));
    if (text.length === 4 && int(text) >= 1900 && int(text) <= 2099) return year(int(text));
    return digits(text);
  }
  function sub(regex, text, fn) {
    let out = "", at = 0;
    for (const m of text.matchAll(regex)) {
      if (m.index < at) continue;
      const match = { group: (n = 0) => m[n], groups: m.groups ?? {}, start: m.index, end: m.index + m[0].length, string: text };
      const result = fn(match);
      const [replacement, extra] = Array.isArray(result) ? result : [result, 0];
      out += text.slice(at, m.index) + replacement;
      at = match.end + extra;
    }
    return out + text.slice(at);
  }
  var escape = (text) => text.replace(/[.*+?^${}()|[\]\\/]/g, "\\$&");
  var Normalizer = class _Normalizer {
    /** isWord(lower case spelling): whether the voice's dictionary knows the word (dizione's is_word). */
    constructor(isWord = () => true) {
      this.isWord = isWord;
      const names = [...new Set([...Object.keys(UNITS), ...Object.keys(EXACT_UNITS)].map(escape))].sort((a, b) => b.length - a.length || (a < b ? -1 : a > b ? 1 : 0));
      this.unitNames = names.join("|");
      this.unit = new RegExp(String.raw`(?<![\w.])(\d[\d,]*(?:\.\d+)?)(?:[-–](\d[\d,]*(?:\.\d+)?))?\s?(${this.unitNames})(?![A-Za-z])`, "gi");
      this.area = /(?<![\w.])(\d[\d,]*(?:\.\d+)?)\s?(km|cm|mm|m|ft|mi|yd|in)([²³])/g;
      this.areaPlain = /(?<![\w.])(\d[\d,]*(?:\.\d+)?)\s?(km|cm|mm|m|ft)([23])(?!\w)/g;
      this.dayFirst = false;
      this.dated = false;
    }
    amount(symbol, whole, fraction, scale) {
      const [one, many, cent, cents] = CURRENCY[symbol];
      if (scale) {
        const said = SCALE_LETTER[scale.toLowerCase()] ?? scale.toLowerCase();
        return `${numberWords(whole + (fraction ? "." + fraction : ""))} ${said} ${many}`;
      }
      if (fraction && (fraction.length > 2 || !cent)) return `${numberWords(whole + "." + fraction)} ${many}`;
      const amount = int(whole), change = int((fraction || "0").padEnd(2, "0"));
      const parts = [];
      if (amount || !change) parts.push(`${cardinal(amount)} ${amount === 1 ? one : many}`);
      if (change && cent) parts.push(`${cardinal(change)} ${change === 1 ? cent : cents}`);
      return parts.join(" and ");
    }
    money(match) {
      const found = match.groups;
      const said = this.amount(found.symbol, found.whole.replace(/,/g, ""), found.fraction, found.scale);
      return (found.minus || found.inner ? " minus " : " ") + said + " ";
    }
    static halfDay(match, dotted, plain) {
      if (!(dotted || plain)) return "";
      const following = match.string.slice(match.end).trimStart();
      const first = following.match(/^[A-Za-z']+/);
      const closes = Boolean(dotted) && match.group(0).endsWith(".") && (!following || isUpper(following[0]) && first !== null && SENTENCE_STARTERS.has(first[0].toLowerCase()));
      return `${SPELL}${(dotted || plain).toUpperCase()}M${END}${closes ? " ." : ""} `;
    }
    time(match) {
      const hour = int(match.group(1)), minute = int(match.group(2)), second = match.group(3);
      const half2 = match.group(4) || match.group(5);
      const list = before(match.string, match.start, 3);
      if (second !== void 0 && !half2 && !(list.length && list[list.length - 1] === "at")) {
        return ` ${duration(hour, minute, int(second))} `;
      }
      if (hour > 24 && second === void 0 && !half2) {
        const after = match.string.slice(match.end).match(/^\s*([A-Za-z]+)/);
        if (list.some((word) => DURATION_WORDS.has(word)) || after && DURATION_WORDS.has(after[1].toLowerCase())) {
          return ` ${duration(0, hour, minute)} `;
        }
        return ` ${cardinal(hour)} ${minute < 10 ? "oh " : ""}${cardinal(minute)} `;
      }
      if (hour > 24) return match.group(0);
      let said = cardinal(hour);
      if (minute === 0) {
        if (half2) {
        } else if (hour === 0) said = "midnight";
        else if (hour > 12) said += " hundred";
        else said += " o'clock";
      } else {
        said += (minute < 10 ? " oh " : " ") + cardinal(minute);
      }
      if (second) said += " and " + cardinal(int(second)) + (int(second) === 1 ? " second" : " seconds");
      return ` ${said} ` + _Normalizer.halfDay(match, match.group(4), match.group(5));
    }
    dotTimes(match) {
      const following = match.string.slice(match.end);
      if (int(match.group(1)) > 23 || int(match.group(4)) > 23 || !before(match.string, match.start).some((word) => CLOCK_WORDS.has(word)) || new RegExp(String.raw`^\s?(?:%|percent|million|billion|${this.unitNames})(?![A-Za-z])`, "i").test(following)) {
        return match.group(0);
      }
      return `${match.group(1)}:${match.group(2)}${match.group(3)}${match.group(4)}:${match.group(5)}`;
    }
    ratio(match) {
      const first = match.group(1), second = match.group(2);
      if (first.length <= 2 && second.length === 2 && "012345".includes(second[0])) {
        const after = match.string.slice(match.end).match(/^\s*([A-Za-z]+)/);
        if (!(before(match.string, match.start, 3).some((word) => RATIO_WORDS.has(word)) || after && RATIO_WORDS.has(after[1].toLowerCase()))) {
          return match.group(0);
        }
      }
      return ` ${numberWords(first)} to ${numberWords(second)} `;
    }
    date(month, day, yearText, the = false) {
      if (!(month >= 1 && month <= 12) || !(day >= 1 && day <= 31)) return null;
      let said = the ? `the ${ordinal(day)} of ${MONTHS[month - 1]}` : `${MONTHS[month - 1]} ${ordinal(day)}`;
      if (yearText) {
        said += " " + (yearText.length === 4 ? year(int(yearText)) : (int(yearText) < 10 ? "oh " : "") + cardinal(int(yearText)));
      }
      return ` ${said} `;
    }
    slashDate(match) {
      let first = int(match.group(1)), second = int(match.group(2));
      if (first > 12 && 12 >= second || this.dayFirst && second <= 12) [first, second] = [second, first];
      return this.date(first, second, match.group(3)) ?? match.group(0);
    }
    dotDate(match) {
      let day = int(match.group(1)), month = int(match.group(2));
      if (month > 12 && 12 >= day) [day, month] = [month, day];
      return this.date(month, day, match.group(3)) ?? match.group(0);
    }
    dayMonth(match) {
      const month = match.group(2);
      const spoken = this.date(MONTH_NUMBER[month.toLowerCase().slice(0, 3)], int(match.group(1)), match.group(4), true);
      if (spoken === null) return match.group(0);
      return spoken + (match.group(3) && MONTHS.includes(month) ? ". " : "");
    }
    daysOfMonth(match) {
      const monthFirst = /\p{L}/u.test(match.group(1)[0]);
      const [month, a, b] = monthFirst ? [match.group(1), match.group(2), match.group(3)] : [match.group(3), match.group(1), match.group(2)];
      const first = int(a), second = int(b);
      if (!(first >= 1 && first <= 31 && second >= 1 && second <= 31)) return match.group(0);
      const name = MONTHS[MONTH_NUMBER[month.toLowerCase().slice(0, 3)] - 1];
      const said = monthFirst ? `${name} ${ordinal(first)} to ${ordinal(second)}` : `the ${ordinal(first)} to the ${ordinal(second)} of ${name}`;
      return ` ${said}${match.group(4) ? " " + year(int(match.group(4))) : ""} `;
    }
    dayRange(match) {
      const names = [match.group(1), match.group(3)].map((name) => {
        const key = name.toLowerCase();
        return key in ABBREVIATIONS ? ABBREVIATIONS[key][1] : DAY_NAMES[key] ?? name;
      });
      const following = match.string.slice(match.end).trimStart();
      const first = following.match(/^[A-Za-z']+/);
      const ends = Boolean(match.group(4)) && (!following || isUpper(following[0]) && first !== null && SENTENCE_STARTERS.has(first[0].toLowerCase()));
      return `${names[0]} to ${names[1]}${ends ? " ." : ""}`;
    }
    shortDate(match) {
      const firstText = match.group(1), secondText = match.group(2);
      if (firstText.length > 2 || secondText.length > 2) return null;
      let first = int(firstText), second = int(secondText);
      const list = before(match.string, match.start);
      if (first >= 1 && first <= 12 && second >= 20 && second <= 49 && list.includes("card")) {
        return ` ${MONTHS[first - 1]} ${year(2e3 + second)} `;
      }
      const last = list.length ? list[list.length - 1] : "";
      const classic = first < second && CLASSIC_FRACTIONS.has(second);
      const zero = firstText.length === 2 && firstText[0] === "0" || secondText.length === 2 && secondText[0] === "0";
      if (!(zero || first > 12 && first <= 31 && second <= 12 && !(first === 24 && second === 7) || DATE_BEFORE.has(last) && (!classic || last === "on") || this.dated && !classic && first !== second)) {
        return null;
      }
      if (first > 12 || this.dayFirst && second <= 12) [first, second] = [second, first];
      return this.date(first, second);
    }
    fraction(match) {
      const spoken = this.shortDate(match);
      if (spoken) return spoken;
      const top = int(match.group(1)), bottom = int(match.group(2));
      if (top === 24 && bottom === 7 || top === bottom) return ` ${cardinal(top)} ${cardinal(bottom)} `;
      if (!(top > 0 && top < 100) || !(bottom > 1 && bottom <= 100)) return ` ${cardinal(top)} over ${cardinal(bottom)} `;
      return ` ${fractionWords(top, bottom)} `;
    }
    mixedFraction(match) {
      const top = int(match.group(2)), bottom = int(match.group(3));
      if (!(top > 0 && top < bottom)) return match.group(0);
      return ` ${numberWords(match.group(1))} and ${fractionWords(top, bottom, true)} `;
    }
    vulgar(match) {
      const [top, bottom] = match.group(2).normalize("NFKC").split("\u2044").map(int);
      if (match.group(1) === void 0) return ` ${top}/${bottom} `;
      return ` ${cardinal(int(match.group(1)))} and ${fractionWords(top, bottom, true)} `;
    }
    phone(match) {
      const groups = [1, 2, 3, 4].map((n) => match.group(n)).filter(Boolean);
      return " " + groups.map((group) => group === "800" ? "eight hundred" : digits(group, "oh")).join(", ") + " ";
    }
    measure(match) {
      const unit = match.group(3);
      const names = EXACT_UNITS[unit] ?? UNITS[unit.toLowerCase()];
      const next = match.string.slice(match.end, match.end + 1);
      if (names === void 0 || unit.length === 1 && ["-", "'", "/"].includes(next)) return match.group(0);
      const [one, many] = names;
      const value = match.group(1), upper = match.group(2);
      const stop = /^\.\s+[a-z]/.test(match.string.slice(match.end)) ? 1 : 0;
      if (upper) return [` ${numberWords(value)} to ${numberWords(upper)} ${many} `, stop];
      return [` ${numberWords(value)} ${value === "1" || value === "1.0" ? one : many} `, stop];
    }
    square(match) {
      const unit = match.group(2);
      const [one, many] = unit === "in" ? ["inch", "inches"] : EXACT_UNITS[unit] ?? UNITS[unit];
      const kind = "\xB22".includes(match.group(3)) ? "square" : "cubic";
      return ` ${numberWords(match.group(1))} ${kind} ${match.group(1) === "1" || match.group(1) === "1.0" ? one : many} `;
    }
    counted(text, end) {
      const after = text.slice(end).match(/^\s*([A-Za-z]+)/);
      if (!after) return false;
      const word = after[1].toLowerCase();
      if (["people", "men", "women", "children", "feet", "miles"].includes(word)) return true;
      return word.length > 3 && word.endsWith("s") && !word.endsWith("ss") && !NOT_COUNTED.has(word) && this.isWord(word.slice(0, -1));
    }
    number(match) {
      const text = match.string, value = match.group(0);
      const plain = value.replace(/,/g, "");
      if (!plain.includes(".") && !value.includes(",") && !plain.startsWith("0") && plain.length > 2) {
        const kind = identifier(text, match.start);
        const limit = { phone: 3, pin: 3, strong: 4, enter: 4, weak: 5 }[kind] ?? 7;
        if (plain.length >= limit && !(kind === null && plain.endsWith("000")) && !(kind === "enter" && plain.length === 4 && int(plain) >= 1900 && int(plain) <= 2099) && !([null, "phone", "weak"].includes(kind) && this.counted(text, match.end))) {
          return ` ${digits(plain, kind === "phone" ? "oh" : "zero")} `;
        }
      }
      if (!plain.includes(".") && !value.includes(",") && plain.length === 4 && !plain.startsWith("0")) {
        const number = int(plain);
        const beforeWords = text.slice(0, match.start).slice(-24).match(/[A-Za-z]+/g) ?? [];
        const after = text.slice(match.end).match(/^\s*([A-Za-z]+)/);
        const counted = Boolean(after) && /(?:[^s]s|people|men|women|children|feet|miles)$/.test(after[1].toLowerCase());
        const lastBefore = beforeWords.length ? beforeWords[beforeWords.length - 1].toLowerCase() : "";
        const dated = Boolean(lastBefore) && (YEAR_CONTEXT.has(lastBefore) || lastBefore.slice(0, 3) in MONTH_NUMBER);
        const model = text.slice(0, match.start).match(/(?<![A-Za-z])([A-Z]{2,5}) $/);
        if (model && !CAPITAL_WORDS.has(model[1]) && !counted) return ` ${pairs(number)} `;
        if (number >= 1100 && number <= 1999 || number >= 2e3 && number <= 2099 && (dated || !counted)) return ` ${year(number)} `;
      }
      return ` ${numberWords(value)} `;
    }
    readsAsWord(letters) {
      const lower = letters.toLowerCase();
      if (lower.length < 3 || !this.isWord(lower)) return false;
      return lower.length > 3 || ["s", "es", "ed", "ing"].some((ending) => this.isWord(lower + ending));
    }
    static alnum(group, zero = "zero") {
      return (group.match(/[A-Za-z]+|\d+/g) ?? []).map((run) => isDigits(run) ? digits(run, zero) : spelled(run)).join(" ");
    }
    static digitGroups(groups, zero = "zero") {
      if (groups.every((group) => shortNumber(group) && group.length <= 2)) return groups.map((group) => cardinal(int(group))).join(" , ");
      return groups.map((group) => digits(group, zero)).join(" , ");
    }
    codeWords(groups, match) {
      const token = groups.join("-");
      if (!/\d/.test(token)) return null;
      const kinds = groups.map((g) => isDigits(g) ? "d" : /\d/.test(g) ? "m" : isUpper(g) ? "u" : "l");
      if (kinds.every((kind) => kind === "d")) {
        if (groups.length === 2) {
          const years = groups.every((g) => g.length === 4 && int(g) >= 1e3 && int(g) <= 2100);
          if (!(match.group(1) || groups.some((g) => g.length > 1 && g[0] === "0") || Math.min(...groups.map((g) => g.length)) >= 4 && !years || ["strong", "pin"].includes(identifier(match.string, match.start)))) {
            return null;
          }
        }
        return _Normalizer.digitGroups(groups);
      }
      if (groups.length === 2 && kinds[1] === "d" && "ul".includes(kinds[0])) {
        const [letters, number] = groups;
        if (kinds[0] === "l" && letters.length > 1 || this.readsAsWord(letters)) return null;
        if (shortNumber(number)) return `${spelled(letters)} ${cardinal(int(number))}`;
        return `${spelled(letters)} , ${digits(number)}`;
      }
      if (groups.length === 2 && kinds[0] === "d" && (kinds[1] === "l" || groups[1].length === 1 || /^\d+[A-Za-z]+$/.test(groups[1]))) {
        return null;
      }
      if (kinds.some((kind, i) => kind === "l" && groups[i].length > 2 || kind === "m" && /^\d+(?:s|st|nd|rd|th)$/.test(groups[i]))) {
        return null;
      }
      if (!/[A-Z]/.test(token) && (kinds.includes("l") || kinds.includes("m"))) return null;
      return groups.map((group, i) => {
        const kind = kinds[i];
        if (kind === "d") return digits(group);
        if (kind === "m") return _Normalizer.alnum(group);
        if (group.length > 3 && this.isWord(group.toLowerCase())) return group;
        return group.length <= 5 ? spelled(group) : group;
      }).join(" , ");
    }
    code(match) {
      const said = this.codeWords(match.group(2).split("-"), match);
      if (said === null) return match.group(0);
      return (match.group(1) ? " number " : " ") + said + " ";
    }
    grouped(match) {
      const text = match.string;
      const plus = match.group(1);
      let lead = match.group(2), trail = match.group(4);
      const core = (match.group(3) ?? match.group(5)).split(" ");
      let head = "", tail = "";
      if (lead && CAPITAL_WORDS.has(lead)) {
        head = lead + " ";
        tail = trail ? " " + trail : "";
        lead = void 0;
        trail = void 0;
      }
      const kind = identifier(text, match.start);
      const pure = core.every(isDigits);
      if (pure && !plus && !lead) {
        const following = text.slice(match.end);
        const amount = this.counted(text, match.end) || new RegExp(String.raw`^\s?(?:${this.unitNames})(?![A-Za-z])`, "i").test(following);
        if (kind === null && core[0][0] !== "0" && core[0].length <= 3 && core.slice(1).every((part) => part.length === 3) && (core[core.length - 1] === "000" || amount && core.length > 2)) {
          return head + core.join(",") + tail;
        }
        if (!(kind || core[0].length > 1 && core[0][0] === "0" || core.length > 2 && core.some((part) => part.length > 2))) {
          return match.group(0);
        }
      } else if (!pure && !lead && !plus && kind === null) {
        return match.group(0);
      }
      const zero = plus || kind === "phone" || pure && core[0][0] === "0" && core.reduce((n, part) => n + part.length, 0) >= 10 ? "oh" : "zero";
      let spoken = pure ? _Normalizer.digitGroups(core, zero) : core.map((part) => isDigits(part) ? digits(part, zero) : _Normalizer.alnum(part, zero)).join(" , ");
      if (lead) spoken = `${spelled(lead)} , ${spoken}`;
      if (trail) spoken = `${spoken} , ${spelled(trail)}`;
      return head + (plus ? " plus " : " ") + spoken + " " + tail;
    }
    glued(match) {
      const out = (match.group(0).match(/[A-Za-z]+|\d+/g) ?? []).map((run) => {
        if (isDigits(run)) return shortNumber(run) ? cardinal(int(run)) : digits(run);
        if (run.length === 1 || (isUpper(run) || run.length <= 3) && !this.readsAsWord(run)) return spelled(run);
        return run;
      });
      return " " + out.join(" ") + " ";
    }
    addressPart(part) {
      const lower = part.toLowerCase();
      if (part.length > 1 && this.isWord(lower) && (part.length > 2 || SHORT_WORDS.has(lower))) return part;
      return part.length <= 3 ? spelled(part) : part;
    }
    spokenAddress(core) {
      const pieces = core.match(/[A-Za-z]+|\d+|./gs) ?? [];
      const mail = core.includes("@");
      let host = false;
      const out = [];
      pieces.forEach((piece, index) => {
        const following = pieces[index + 1] ?? "";
        if (piece === "@") host = true;
        if (isDigits(piece)) out.push(addressDigits(piece));
        else if (!isAlpha(piece)) out.push(SEPARATORS[piece] ?? "");
        else if (index && pieces[index - 1] === "." && (host || !mail) && (["", "/", ":", "?", "#"].includes(following) || piece.length <= 3 && (piece.toLowerCase() in ENDINGS || !this.isWord(piece.toLowerCase())))) {
          out.push(ENDINGS[piece.toLowerCase()] === "word" ? piece.toLowerCase() : spelled(piece));
        } else {
          let letters = "";
          for (const part of piece.match(/[A-Z]?[a-z]+|[A-Z]+(?![a-z])/g) ?? []) {
            const spoken = this.addressPart(part);
            if (spoken.startsWith(SPELL)) {
              letters += part;
              continue;
            }
            if (letters) {
              out.push(spelled(letters));
              letters = "";
            }
            out.push(spoken);
          }
          if (letters) out.push(spelled(letters));
        }
      });
      return out.filter(Boolean).join(" ");
    }
    address(match) {
      const whole = match.group(0);
      const text = whole.replace(/[.,;:!?)]+$/, "");
      const tail = whole.slice(text.length);
      if (match.groups.file) {
        const ending = match.groups.ending;
        const known = ending.toLowerCase() in ENDINGS;
        const marked = Boolean(match.groups.folder || match.groups.path) || match.groups.stem.includes("_");
        if (!(known || marked && ending.length <= 5 && (ending === ending.toLowerCase() || isUpper(ending)) && !this.isWord(ending.toLowerCase()))) {
          return whole;
        }
      }
      const core = text.replace(/^(?:https?:\/\/)?(?:www\.)?/, "").replace(/\/+$/, "");
      return ` ${this.spokenAddress(core)} ${tail}`;
    }
    chatForm(match) {
      const said = CHAT[match.group(0).toLowerCase()];
      if (said === void 0) return match.group(0);
      if (said === "-") return " ";
      return " " + said.split(" ").map((part) => part.startsWith("*") ? spelled(part.slice(1)) : part).join(" ") + " ";
    }
    dotted(match) {
      const letters = match.group(0).replace(/\./g, "");
      const key = letters.toLowerCase();
      const following = match.string.slice(match.end).trimStart();
      const said = key === "eg" ? "for example" : key === "ie" ? "that is" : spelled(letters.toUpperCase());
      const first = following.match(/^[A-Za-z']+/);
      const endsSentence = !following || isUpper(following[0]) && first && SENTENCE_STARTERS.has(first[0].toLowerCase());
      return ` ${said}${endsSentence && match.group(0).endsWith(".") ? " ." : ""} `;
    }
    abbreviation(match) {
      const text = match.string, token = match.group(1), stop = match.group(2);
      const key = token.toLowerCase();
      const following = text.slice(match.end);
      const nextWord = following.match(/^\s*([A-Za-z]+|\d)/);
      const capitalNext = Boolean(nextWord) && isUpper(nextWord[1][0]);
      const capitalBefore = /[A-Z][a-z]+\s+$/.test(text.slice(0, match.start));
      if ((key === "st" || key === "dr") && isUpper(token[0])) {
        if (capitalNext && (stop || key === "dr")) return key === "st" ? " saint " : " doctor ";
        if (capitalBefore && stop) return (key === "st" ? " street" : " drive") + (!following.trim() ? " ." : " ");
        return match.group(0);
      }
      if (!(key in ABBREVIATIONS)) return match.group(0);
      let [when, said] = ABBREVIATIONS[key];
      const numberNext = Boolean(nextWord) && isDigits(nextWord[1]);
      if (when === "title") {
        if (!(isUpper(token[0]) && (capitalNext || stop))) return match.group(0);
      } else if (when === "number") {
        const previous = text.slice(0, match.start).match(/([A-Za-z]+)\.?\s+$/);
        if (!(stop && (numberNext || key === "no" && previous && NUMBERED.has(previous[1].toLowerCase())))) return match.group(0);
      } else if (!stop || when === "small" && isUpper(token)) {
        return match.group(0);
      } else if (["mar", "sun", "sat", "jan", "dec", "max", "min", "est", "no"].includes(key) && !isUpper(token[0]) && !numberNext) {
        if (!(["max", "min", "est"].includes(key) && nextWord && nextWord[1][0] === nextWord[1][0].toLowerCase() && nextWord[1][0] !== nextWord[1][0].toUpperCase())) {
          return match.group(0);
        }
      }
      if (key === "est" && !numberNext) said = "estimated";
      const endsSentence = when !== "title" && (!following.trim() || capitalNext && SENTENCE_STARTERS.has(nextWord[1].toLowerCase()));
      return ` ${said}${endsSentence ? " ." : ""} `;
    }
    /** dizione's Normalizer.rewrite: the text with every non-word written out, letters to name between SPELL and END. */
    rewrite(text) {
      text = sub(VULGAR, text, (m) => this.vulgar(m));
      text = sub(this.area, text, (m) => this.square(m));
      text = text.replace(/²/g, " squared ").replace(/³/g, " cubed ");
      text = text.normalize("NFKC").replace(/⁄/g, "/").replace(/−/g, " minus ");
      text = text.replace(/(?<=\d)–(?=\d)/g, "-");
      text = sub(DAY_RANGE, text, (m) => this.dayRange(m));
      for (const [old, replacement] of [
        ["\u2018", "'"],
        ["\u2019", "'"],
        ["\u201A", "'"],
        ["\u201B", "'"],
        ["`", "'"],
        ["\xB4", "'"],
        ["\u201C", '"'],
        ["\u201D", '"'],
        ["\u201E", '"'],
        ["\u201F", '"'],
        ["\xAB", '"'],
        ["\xBB", '"'],
        ["\u2013", "\u2014"],
        ["\u2015", "\u2014"],
        ["[", "("],
        ["]", ")"],
        ["{", "("],
        ["}", ")"]
      ]) {
        text = text.split(old).join(replacement);
      }
      text = sub(ADDRESS, text, (m) => this.address(m));
      text = text.replace(/→/g, " to ");
      text = text.replace(EMOJI, " ").replace(EMOTICON, " ");
      text = text.replace(/\.{3,}/g, "\u2026");
      text = text.replace(/(?<=\d)\s-\s(?=\d[\d.,]*\s?=)/g, " minus ");
      text = text.replace(/-{2,}|\s-\s/g, "\u2014");
      text = text.replace(/\bw\/o\b/gi, " without ").replace(/\bw\/(?=\s|$)/gi, " with ").replace(/\bn\/a\b/gi, " not applicable ");
      text = text.split("'n'").join(" 'n ");
      if (/\d/.test(text)) text = this.figures(text);
      return this.signs(text);
    }
    figures(text) {
      text = sub(MONEY, text, (m) => this.money(m));
      text = sub(MONEY_AFTER, text, (m) => this.money(m));
      text = text.replace(/(?<![\w.])(\d[\d,]*(?:\.\d+)?)\s?¢/g, (_, n) => ` ${numberWords(n)} ${n === "1" ? "cent" : "cents"} `);
      const clock = String.raw`\d{1,2}:[0-5]\d(?:\s?[AaPp]\.?[Mm]\.?)?|\b\d{1,2}\s?[AaPp]\.?[Mm]\.?`;
      text = text.replace(new RegExp(`(${clock})\\s(${ZONES})\\b`, "g"), (_, time, zone) => `${time} ${spelled(zone)}`);
      text = text.replace(/(?<![\d.,:])(\d{1,2})\.([0-5]\d)(?=\s?[AaPp]\.?[Mm]\b)/g, "$1:$2");
      text = sub(
        /(?<![\d.,:])(\d{1,2})\.(00|15|30|45)(\s?(?:to|until|till|and|[-—])\s?)(\d{1,2})\.(00|15|30|45)(?!\d|\.\d)/g,
        text,
        (m) => this.dotTimes(m)
      );
      text = text.replace(new RegExp(String.raw`(${clock})\s?[-—]\s?(?=\d{1,2}(?::[0-5]\d|\s?[AaPp]\.?[Mm]\b))`, "g"), "$1 to ");
      text = sub(RATIO, text, (m) => this.ratio(m));
      text = sub(new RegExp(String.raw`(?<![\d:])(\d{1,2}):([0-5]\d)(?::([0-5]\d))?${HALF_DAY}?(?![\d:])`, "g"), text, (m) => this.time(m));
      text = sub(
        new RegExp(String.raw`(?<![\d.:,])(\d{1,2})${HALF_DAY}(?![A-Za-z])`, "g"),
        text,
        (m) => ` ${cardinal(int(m.group(1)))} ` + _Normalizer.halfDay(m, m.group(2), m.group(3))
      );
      const seen = [...text.matchAll(/(?<![\d/.])(\d{1,2})\/(\d{1,2})(?:\/(?:\d{4}|\d{2}))?(?![\d/])/g)].map((m) => [int(m[1]), int(m[2])]).filter(([a, b]) => !(a === 24 && b === 7));
      this.dayFirst = seen.some(([a, b]) => a > 12 && 12 >= b) && !seen.some(([a, b]) => b > 12 && 12 >= a);
      this.dated = /(?<![\d/])\d{1,2}\/\d{1,2}\/(?:\d{4}|\d{2})(?![\d/])/.test(text);
      text = sub(
        /(?<![\d-])(\d{4})-(\d{1,2})-(\d{1,2})(?![\d-])/g,
        text,
        (m) => this.date(int(m.group(2)), int(m.group(3)), m.group(1)) ?? m.group(0)
      );
      text = sub(
        /(?<![\d/])(\d{4})\/(\d{1,2})\/(\d{1,2})(?![\d/])/g,
        text,
        (m) => this.date(int(m.group(2)), int(m.group(3)), m.group(1)) ?? m.group(0)
      );
      text = sub(/(?<![\d.])(\d{1,2})\.(\d{1,2})\.(\d{4})(?!\d|\.\d)/g, text, (m) => this.dotDate(m));
      text = sub(/(?<![\d/])(\d{1,2})\/(\d{1,2})\/(\d{4}|\d{2})(?![\d/])/g, text, (m) => this.slashDate(m));
      text = sub(
        new RegExp(String.raw`\b(${MONTH_PATTERN})\.?\s+(\d{1,2})(?:st|nd|rd|th)?\s?[-—]\s?(\d{1,2})(?:st|nd|rd|th)?(?:,?\s+(\d{4}))?(?![\d:])`, "g"),
        text,
        (m) => this.daysOfMonth(m)
      );
      text = sub(
        new RegExp(String.raw`(?<![\d.,/:-])(\d{1,2})(?:st|nd|rd|th)?\s?[-—]\s?(\d{1,2})(?:st|nd|rd|th)?\s+(${MONTH_PATTERN})\b(?:,?\s+(\d{4}))?`, "g"),
        text,
        (m) => this.daysOfMonth(m)
      );
      text = sub(
        new RegExp(String.raw`\b(${MONTH_PATTERN})\.?\s+(\d{1,2})(?:st|nd|rd|th)?(?:,?\s+(\d{4}))?(?![\d:])`, "g"),
        text,
        (m) => this.date(MONTH_NUMBER[m.group(1).toLowerCase().slice(0, 3)], int(m.group(2)), m.group(3)) ?? m.group(0)
      );
      text = sub(
        new RegExp(String.raw`(?<![\d.,/:-])(\d{1,2})(?:st|nd|rd|th)?\s+(${MONTH_PATTERN})\b(\.(?=\s+[a-z\d]))?(?:,?\s+(\d{4}))?`, "g"),
        text,
        (m) => this.dayMonth(m)
      );
      text = sub(/(?<![\d-])(?:\+?(1)[-.\s])?\(?(\d{3})\)?[-.\s](\d{3})[-.\s](\d{4})(?![\d-])/g, text, (m) => this.phone(m));
      text = sub(/(?<![\w.,-])(\d{3})-(\d{4})(?![\w-])/g, text, (m) => this.phone(m));
      text = sub(
        /(?<![\w.])([vV])?(\d+(?:\.\d+){2,})(?:-(\d+)(?![\w-]|\.\d))?(?!\w)/g,
        text,
        (m) => " " + (m.group(1) ? spelled(m.group(1)) + " " : "") + m.group(2).split(".").map(versionPart).join(" point ") + (m.group(3) ? " dash " + numberWords(m.group(3)) : "") + " "
      );
      text = sub(
        /(?<![\w.])([vV])(\d+)\.(\d{2,3})(?!\d|\.\d|\w)/g,
        text,
        (m) => ` ${spelled(m.group(1))} ${cardinal(int(m.group(2)))} point ${minor(m.group(3))} `
      );
      text = sub(
        new RegExp(String.raw`\b(${VERSIONED})(\.?\s(?:(?:from|to)\s)?)(\d+)\.(\d{2,3})(?:(\s(?:to|or|and)\s)(\d+)\.(\d{2,3}))?(?!\d|\.\d|\s?%|\s(?:percent|million|billion)\b)`, "gi"),
        text,
        (m) => `${m.group(1)}${m.group(2)}${cardinal(int(m.group(3)))} point ${minor(m.group(4))}` + (m.group(5) ? `${m.group(5)}${cardinal(int(m.group(6)))} point ${minor(m.group(7))}` : "") + " "
      );
      text = sub(CODE, text, (m) => this.code(m));
      text = sub(SLASHED, text, (m) => " " + _Normalizer.digitGroups(m.group(1).split("/")) + " ");
      text = sub(GROUPED, text, (m) => this.grouped(m));
      text = text.replace(/(?<![\w.])-(?=\d)/g, " minus ");
      text = text.replace(new RegExp(String.raw`(?:(?<=\d)\s?[xX×]|×)\s?(?=\d[\d.,]*\s?(?:${LENGTHS})\b)`, "g"), " by ");
      text = text.replace(/(?<=\d)\s[xX×]\s(?=\d)/g, " times ");
      text = sub(/(?<![\w/'])[A-Za-z0-9/]*[0-9/][A-Za-z0-9/]*(?![\w/'])/g, text, (m) => this.chatForm(m));
      text = text.replace(/(?<![\w.])'?(\d{2}|\d{4})'?s\b/g, (_, n) => " " + pluralOf(n.length === 4 ? year(int(n)) : cardinal(int(n))) + " ");
      text = text.replace(/(?<![\w.])(\d+)(?:st|nd|rd|th)\b/g, (_, n) => ` ${ordinal(int(n))} `);
      text = text.replace(
        /(?<![\w.])(144|240|360|480|540|576|720|900|1080|1440|2160|4320)([pi])\b/g,
        (_, n, letter) => ` ${pairs(int(n))} ${spelled(letter)} `
      );
      text = text.replace(/(\d[\d,]*(?:\.\d+)?)\s?%/g, (_, n) => ` ${numberWords(n)} percent `);
      text = sub(this.areaPlain, text, (m) => this.square(m));
      text = sub(this.unit, text, (m) => this.measure(m));
      text = sub(/(?<![\d/.,-])(\d+) (\d)\/(2|3|4|5|6|8|10|16)(?![\d/]|\.\d)/g, text, (m) => this.mixedFraction(m));
      text = sub(/(?<![\d/.])(\d+)\/(\d+)(?![\d/]|\.\d)/g, text, (m) => this.fraction(m));
      text = text.replace(/(?<![\w.,-])(\d{4})[-–](\d{4})(?![\w.,-])/g, (_, a, b) => ` ${year(int(a))} to ${year(int(b))} `);
      text = text.replace(/(?<![\w.,-])(\d+)[-–](\d+)(?![\w-]|[.,]\d)/g, (_, a, b) => ` ${numberWords(a)} to ${numberWords(b)} `);
      text = text.replace(/(?<![\w.])(\d+)x\b/g, (_, n) => ` ${numberWords(n)} times `);
      text = text.replace(/(?<=\d)\^(?=\d)/g, " to the power of ");
      return text;
    }
    signs(text) {
      text = text.replace(new RegExp(String.raw`(?<![\w-])(${SIZES})-(${SIZES})(?![\w-])`, "g"), (_, a, b) => ` ${spelled(a)} to ${spelled(b)} `);
      text = text.replace(/#(?=\d)/g, " number ").replace(/#(?=[A-Z]{1,4}-?\d)/g, " number ").replace(/#(?=[A-Za-z])/g, " hashtag ");
      text = text.replace(/~(?=\s?\d)/g, " about ");
      text = text.replace(/(?<=[\w\s])(>=|<=|!=)(?=[\w\s])/g, (_, sign) => ` ${COMPARISONS[sign]} `);
      text = text.replace(/(?<=\d) Ti\b/g, ` ${spelled("TI")}`);
      text = text.replace(/\b([Rr])ef\.\s?[Nn]o\.(?=\s)/g, "$1eference number");
      if (text.includes(".")) text = sub(/\b((?:[A-Za-z]\.){2,})/g, text, (m) => this.dotted(m));
      text = sub(/\b([A-Za-z]{1,6})\b(\.?)(?![A-Za-z'’])/g, text, (m) => this.abbreviation(m));
      if (/\d/.test(text)) {
        text = sub(GLUED, text, (m) => this.glued(m));
        text = sub(/(?<![\w.])\d{1,3}(?:,\d{3})+(?:\.\d+)?|\d+(?:\.\d+)?|(?<![\w.])\.\d+/g, text, (m) => this.number(m));
      }
      text = text.replace(/(?<=[A-Za-z])\/(?=[A-Za-z])/g, " ");
      text = text.replace(/(?<=\d)\s?[<>]\s?(?=\d)|(?<=\s)[<>](?=\s)/g, (sign) => sign.includes("<") ? " less than " : " greater than ");
      text = text.replace(new RegExp(`[${escape(Object.keys(SYMBOL_WORDS).join(""))}]`, "g"), (sign) => ` ${SYMBOL_WORDS[sign]} `);
      return text;
    }
  };
  function spokenText(text, normalizer2, spell) {
    return normalizer2.rewrite(text).replace(new RegExp(`${SPELL}([A-Za-z ]+)${END}`, "g"), (_, letters) => ` ${spell(letters.replace(/ /g, ""))} `).replace(/\s+/g, " ").replace(/ (?=[,.;:!?…])/g, "").replace(/([,.;:!?…])(?=[,.;:!?…])/g, (mark, _, at, all) => mark === all[at + 1] ? "" : mark).trim();
  }

  // ../frontend/dizione/homographs.data.js
  var homographs_data_default = { "plural_bases": ["abstract", "addict", "advocate", "affiliate", "aggregate", "alternate", "animate", "appropriate", "approximate", "articulate", "associate", "attribute", "blessed", "buffet", "combine", "compact", "compound", "conduct", "conflict", "conglomerate", "conjugate", "conscript", "console", "consort", "construct", "consummate", "contest", "contract", "contrast", "converse", "convert", "convict", "coordinate", "decrease", "defect", "degenerate", "delegate", "deliberate", "desert", "deviate", "discard", "discharge", "discount", "document", "duplicate", "elaborate", "entrance", "escort", "estimate", "expatriate", "exploit", "export", "expose", "extract", "fragment", "frequent", "graduate", "impact", "implant", "implement", "import", "incense", "incline", "increase", "increment", "initiate", "insert", "instrument", "insult", "interchange", "intimate", "invert", "invite", "laminate", "learned", "moderate", "moped", "object", "ornament", "overthrow", "perfect", "perfume", "permit", "pervert", "pigment", "postulate", "precipitate", "predicate", "present", "produce", "project", "protest", "rebel", "record", "recount", "refund", "refuse", "reject", "resume", "retard", "separate", "subject", "subordinate", "supplement", "suspect", "syndicate", "tear", "transplant", "transport", "upset"], "pooled": { "bias": -2.0459, "weights": { "cap": -2.3791, "first": -13e-4, "tn1=ADV": 0.0326, "tn1=BE": -1.9895, "tn1=CAP": 0.829, "tn1=CONJ": -0.3262, "tn1=DET": 2.2059, "tn1=ED": -0.5811, "tn1=HAVE": -0.5379, "tn1=LY": 1.5112, "tn1=OBJ": 7.4895, "tn1=POS": 2.2855, "tn1=POSS": 5.0781, "tn1=PREP": -0.9661, "tn1=PUNCT": -0.699, "tn1=S": -0.5612, "tn1=SUBJ": 1.5861, "tn1=TO": -0.518, "tn1=W": -0.2529, "tn1n2=CAP+CAP": -1.067, "tn1n2=CAP+PREP": 0.2951, "tn1n2=CAP+PUNCT": -0.0392, "tn1n2=CAP+TO": 0.244, "tn1n2=CAP+W": -0.6209, "tn1n2=CONJ+W": -0.8776, "tn1n2=DET+S": 0.4925, "tn1n2=DET+SUBJ": -0.3513, "tn1n2=DET+W": 1.5442, "tn1n2=ED+PREP": -0.3539, "tn1n2=ING+PUNCT": -0.4339, "tn1n2=LY+PREP": 0.4924, "tn1n2=PREP+DET": 0.3162, "tn1n2=PREP+S": -0.7325, "tn1n2=PUNCT+CAP": 0.0839, "tn1n2=PUNCT+CONJ": -0.1019, "tn1n2=PUNCT+ED": 0.0468, "tn1n2=PUNCT+PREP": -0.1972, "tn1n2=PUNCT+W": -0.2283, "tn1n2=S+DET": -0.2897, "tn1n2=S+PUNCT": -0.1665, "tn1n2=S+TO": 0.4195, "tn1n2=S+W": -0.2766, "tn1n2=SUBJ+PUNCT": 0.2412, "tn1n2=TO+DET": -0.7581, "tn1n2=W+DET": 0.843, "tn1n2=W+PUNCT": -0.2504, "tn1n2=W+S": 0.6466, "tn1n2=W+TO": -0.0649, "tn1n2=W+W": 0.6605, "tn2=ADV": 0.2338, "tn2=BE": -1.5298, "tn2=ED": -0.537, "tn2=HAVE": -0.6655, "tn2=LY": -0.1787, "tn2=MODAL": -1.4938, "tn2=POS": 0.5558, "tn2=PUNCT": -0.4414, "tn2=S": 0.2707, "tn2=SUBJ": -0.5764, "tn2=TO": 0.4385, "tn2=W": 0.2501, "tp1=<s>": -0.5995, "tp1=ADV": 0.5648, "tp1=BE": -1.5174, "tp1=CONJ": 0.5358, "tp1=DET": -3.0969, "tp1=DO": 1.2311, "tp1=ED": -1.2476, "tp1=HAVE": -0.8374, "tp1=ING": -2.0053, "tp1=LY": 1.1058, "tp1=MODAL": 6.171, "tp1=NEG": 0.7247, "tp1=NUM": -0.7705, "tp1=POS": -1.4653, "tp1=POSS": -3.4859, "tp1=PREP": -2.7183, "tp1=S": 1.4298, "tp1=SUBJ": 4.7497, "tp1=TO": 4.4197, "tp1=W": -1.6407, "tp1n1=<s>+CAP": -0.5351, "tp1n1=<s>+PUNCT": 0.0968, "tp1n1=ADV+PREP": 0.4385, "tp1n1=CAP+CAP": 0.1395, "tp1n1=CAP+PUNCT": -0.3045, "tp1n1=CONJ+CAP": 0.2994, "tp1n1=CONJ+PREP": -0.0389, "tp1n1=CONJ+S": 89e-4, "tp1n1=CONJ+SUBJ": 0.0221, "tp1n1=CONJ+W": -0.2556, "tp1n1=DET+CONJ": -0.9335, "tp1n1=DET+DET": 0.4123, "tp1n1=DET+PREP": -0.3091, "tp1n1=DET+S": 0.722, "tp1n1=LY+PREP": 1.0064, "tp1n1=LY+PUNCT": -0.6799, "tp1n1=PUNCT+PREP": -0.0484, "tp1n1=PUNCT+PUNCT": 0.0138, "tp1n1=PUNCT+S": -0.3554, "tp1n1=PUNCT+W": -0.828, "tp1n1=S+PREP": 0.0226, "tp1n1=S+PUNCT": -0.8279, "tp1n1=S+S": 0.278, "tp1n1=TO+CONJ": 0.6379, "tp1n1=TO+PREP": 0.3896, "tp1n1=W+CONJ": 0.1217, "tp1n1=W+DET": -0.2229, "tp1n1=W+PREP": -0.172, "tp1n1=W+PUNCT": -1.2626, "tp1n1=W+TO": 0.0798, "tp2=<s>": -0.4257, "tp2=ADV": 0.4309, "tp2=BE": -0.5828, "tp2=CAP": 0.1227, "tp2=CONJ": 0.3637, "tp2=DET": -1.5678, "tp2=DO": 0.9146, "tp2=ED": -0.3802, "tp2=HAVE": -0.3374, "tp2=ING": -0.4404, "tp2=MODAL": 2.9652, "tp2=NUM": -0.994, "tp2=PREP": -0.4665, "tp2=PUNCT": -86e-4, "tp2=S": 0.6809, "tp2=SUBJ": 0.7849, "tp2=TO": 1.1335, "tp2=W": -0.0177, "tp2p1=<s>+<s>": -0.0558, "tp2p1=<s>+S": 2.2948, "tp2p1=ADV+PREP": 2.5939, "tp2p1=BE+ADV": -0.1237, "tp2p1=CAP+CAP": 0.0169, "tp2p1=CAP+S": -0.6684, "tp2p1=CONJ+CAP": 0.8386, "tp2p1=DET+ADV": -1.5337, "tp2p1=DET+LY": -0.051, "tp2p1=DET+PUNCT": -1.2407, "tp2p1=DET+S": 1.4817, "tp2p1=DET+W": 0.8899, "tp2p1=DO+NEG": 3.196, "tp2p1=HAVE+W": -0.4077, "tp2p1=POSS+S": 0.2465, "tp2p1=PREP+CAP": -0.0257, "tp2p1=PREP+DET": -0.7828, "tp2p1=PUNCT+DET": 0.6797, "tp2p1=PUNCT+W": -0.1968, "tp2p1=S+ADV": 1.7247, "tp2p1=S+DET": 2.1968, "tp2p1=S+LY": 0.3408, "tp2p1=SUBJ+BE": -0.0894, "tp2p1=TO+DET": -0.185, "tp2p1=TO+LY": 0.2195, "tp2p1=W+PUNCT": -0.6525, "tp2p1=W+TO": -0.4806, "tp2p1=W+W": -0.3634 } }, "words": { "abstract": { "model": { "bias": -1.4232, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "adj-nou", "phones": "\u02C8\xE6 b s t \u0279 \xE6 k t", "role": "n" }, { "id": "vrb", "phones": "\xE6 b s t \u0279 \u02C8\xE6 k t", "role": "v" }] }, "abuse": { "model": { "bias": -0.2386, "kind": "noun-verb", "weights": { "tn1=PREP": -0.0354, "w=of": -0.125 } }, "senses": [{ "id": "nou", "phones": "\u0250 b j \u02C8u\u02D0 s", "role": "n" }, { "id": "vrb", "phones": "\u0250 b j \u02C8u\u02D0 z", "role": "v" }] }, "abuses": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn1=CAP": 0.4892, "tn1=PREP": -0.1999, "tp1=S": -0.7026, "w=her": 0.8644, "w=in": -2.7782, "w=of": -0.2066, "w=on": 0.5553 } }, "senses": [{ "id": "nou", "phones": "\u0250 b j \u02C8u\u02D0 s \u1D7B z", "role": "n" }, { "id": "vrb", "phones": "\u0250 b j \u02C8u\u02D0 z \u1D7B z", "role": "v" }] }, "addict": { "model": { "bias": 0, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "\u02C8\xE6 d \u026A k t", "role": "n" }, { "id": "vrb", "phones": "\u0250 d \u02C8\u026A k t", "role": "v" }] }, "advocate": { "model": { "bias": 0.4232, "kind": "noun-verb", "weights": { "n1=for": 0.7444, "n1=of": -0.4639, "tn1=W": 0.4732, "tp2p1=W+CONJ": 1.0504, "w=such": 0.286 } }, "senses": [{ "id": "nou", "phones": "\u02C8\xE6 d v \u0259 k \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u02C8\xE6 d v \u0259 k \u02CCe\u026A t", "role": "v" }] }, "affect": { "model": { "bias": [0, 2.1135], "kind": "senses", "weights": [{}, { "tn1=DET": 0.8624, "tn1=W": 0.9653, "tn2=W": 1.32, "w=positive": -1.064, "w=processing": -1.0641, "w=to": -0.5173, "w=with": -1.5969 }] }, "senses": [{ "id": "nou-psy", "phones": "\u02C8\xE6 f \u025B k t", "role": "o" }, { "id": "affect", "phones": "\u0250 f \u02C8\u025B k t", "role": "o" }] }, "affiliate": { "model": { "bias": -0.2702, "kind": "noun-verb", "weights": { "tn1=CAP": -0.0366, "tp1=CAP": -1.4108 } }, "senses": [{ "id": "nou", "phones": "\u0250 f \u02C8\u026A l \u026A \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u0250 f \u02C8\u026A l \u026A \u02CCe\u026A t", "role": "v" }] }, "aged": { "model": { "bias": [0, 1.8989], "kind": "senses", "weights": [{}, { "p2=,": -0.4261, "tn1=NUM": 1.8997, "tn2=W": -1.6882, "tp1=PUNCT": 1.0739, "tp2=PUNCT": -0.0192 }] }, "senses": [{ "id": "adj", "phones": "\u02C8e\u026A d\u0292 \u026A d", "role": "n" }, { "id": "aged", "phones": "\u02C8e\u026A d\u0292 d", "role": "o" }] }, "aggregate": { "model": { "bias": 0.1582, "kind": "noun-verb", "weights": { "tn1=PREP": 0.0221, "tn1=PUNCT": 1.6356, "w=on": -0.6165 } }, "senses": [{ "id": "adj-nou", "phones": "\u02C8\xE6 \u0261 \u0279 \u0259 \u0261 \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u02C8\xE6 \u0261 \u0279 \u0259 \u0261 e\u026A t", "role": "v" }] }, "alternate": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn1=PREP": 1.9202, "tn1=S": -0.5842, "tn1=W": -0.5308 } }, "senses": [{ "id": "adj-nou", "phones": "\u02C8\u0254\u02D0 l t \u025A n \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u02C8\u0254\u02D0 l t \u025A n \u02CCe\u026A t", "role": "v" }] }, "analyses": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn1=PREP": -0.0544, "tn1=W": 0.2106, "tn2=S": 0.4328, "tp1=W": 76e-4, "tp2p1=PUNCT+DET": 0.1538, "w=and": 0.5906, "w=of": -0.9975, "w=the": 1.9568, "w=to": -1.303 } }, "senses": [{ "id": "nou", "phones": "\u0250 n \u02C8\xE6 l \u026A s \u02CCi\u02D0 z", "role": "n" }, { "id": "vrb", "phones": "\u02C8\xE6 n \u0259 l \u02CCa\u026A z \u1D7B z", "role": "v" }] }, "animate": { "model": { "bias": 0.756, "kind": "noun-verb", "weights": { "n2=.": 0.2447, "tp1n1=PUNCT+PUNCT": 2.8412, "tp2=CONJ": 0.2856, "w=and": 1.4201, "w=inanimate": -1.0124, "w=nouns": -2.7755, "w=the": 0.0564 } }, "senses": [{ "id": "adj-nou", "phones": "\u02C8\xE6 n \u026A m \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u02C8\xE6 n \u026A m \u02CCe\u026A t", "role": "v" }] }, "appropriate": { "model": { "bias": -1.2222, "kind": "noun-verb", "weights": { "w=and": -0.6521, "w=the": -0.4891 } }, "senses": [{ "id": "adj", "phones": "\u0250 p \u0279 \u02C8o\u028A p \u0279 \u026A \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u0250 p \u0279 \u02C8o\u028A p \u0279 \u026A \u02CCe\u026A t", "role": "v" }] }, "approximate": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "w=to": 0.5854 } }, "senses": [{ "id": "adj-nou", "phones": "\u0250 p \u0279 \u02C8\u0251\u02D0 k s \u026A m \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u0250 p \u0279 \u02C8\u0251\u02D0 k s \u026A m \u02CCe\u026A t", "role": "v" }] }, "articulate": { "model": { "bias": 0.2485, "kind": "noun-verb", "weights": { "tn1=DET": 1.5659, "tn2=S": 0.1389, "tp1=W": 0.938, "tp2=DET": 0.3182, "w=rather": 1.1517, "w=the": 0.1256, "w=to": 1.4538 } }, "senses": [{ "id": "adj", "phones": "\u0251\u02D0\u0279 t \u02C8\u026A k j \u028A l \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u0251\u02D0\u0279 t \u02C8\u026A k j \u028A l \u02CCe\u026A t", "role": "v" }] }, "associate": { "model": { "bias": -0.0377, "kind": "noun-verb", "weights": { "p1=to": -0.0505, "tp1=TO": -1e-4, "w=to": -1.1947, "w=with": 3.3409 } }, "senses": [{ "id": "adj-nou", "phones": "\u0250 s \u02C8o\u028A s \u026A \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u0250 s \u02C8o\u028A s \u026A \u02CCe\u026A t", "role": "v" }] }, "attribute": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn1=DET": 0.1997, "w=of": 0.6248, "w=people": 0.9925, "w=to": 2.9715 } }, "senses": [{ "id": "nou", "phones": "\u02C8\xE6 t \u0279 \u026A b j \u02CCu\u02D0 t", "role": "n" }, { "id": "vrb", "phones": "\u0250 t \u0279 \u02C8\u026A b j u\u02D0 t", "role": "v" }] }, "august": { "model": { "bias": [-1.0989, 0, -0.9983], "kind": "senses", "weights": [{ "cap": -3.3198 }, { "cap": 2.0546, "p1=in": 0.2391, "tn1=CAP": -0.7363, "tn1=NUM": 0.9208, "tn1=PUNCT": 0.6581, "tn1=W": -1.6522, "tp1=CAP": -2.5493, "tp1=PREP": 0.8633, "tp1=W": -0.1918, "w=a": -0.5888, "w=to": 0.099 }, { "cap": -0.9461, "p1=in": -0.3846, "tn1=CAP": 0.4667, "tn1=NUM": -0.3512, "tn1=PUNCT": -0.7681, "tn1=W": 0.2433, "tp1=CAP": 2.3558, "tp1=PREP": -0.7542, "tp1=W": 0.6002, "w=a": 0.5688, "w=is": 0.5984, "w=to": -0.1947 }] }, "senses": [{ "id": "adj", "phones": "\u0254\u02D0 \u0261 \u02C8\u028C s t", "role": "n" }, { "id": "august", "phones": "\u02C8\u0254\u02D0 \u0261 \u0259 s t", "role": "o" }, { "id": "nam", "phones": "\u02C8a\u028A \u0261 \u028A s t", "role": "o" }] }, "axes": { "model": { "bias": [0, 0], "kind": "senses", "weights": [{}, { "n1=,": 1.9447, "n1=.": -2e-4, "n1=of": -0.9912, "n2=the": -0.4257, "p1=with": -0.5326, "tn1=CONJ": 1.6081, "tn1=PREP": -0.5018, "tn1n2=PUNCT+</s>": -0.3212, "tn1n2=W+PREP": 0.0485, "tn2=PREP": 0.1236, "tn2=PUNCT": -0.128, "tn2=W": -0.2739, "tp1=W": -0.0554, "tp1n1=W+PUNCT": -1.043, "tp2=PREP": -0.3958, "tp2p1=DET+W": -0.6666, "tp2p1=PUNCT+W": -0.0423, "w=a": -0.5673, "w=and": 0.4178, "w=are": -1.0853, "w=battle": 0.4046, "w=been": 1.1459, "w=found": 0.3953, "w=hand": 1.4434, "w=of": -0.0181, "w=stone": 0.7517, "w=three": -84e-4, "w=were": 0.4546, "w=with": -0.2894 }] }, "senses": [{ "id": "nou", "phones": "\u02C8\xE6 k s i\u02D0 z", "role": "n" }, { "id": "nou-vrb", "phones": "\u02C8\xE6 k s \u1D7B z", "role": "o" }] }, "bass": { "model": { "bias": [0, -2.463], "kind": "senses", "weights": [{}, { "cap": 0.4053, "n1=,": 1.0512, "tn1=W": -0.8322, "tn2=W": 3.7946, "tp1n1=W+PUNCT": 0.9143, "tp2p1=CONJ+W": 1.1261, "w=on": -0.2952 }] }, "senses": [{ "id": "bass", "phones": "b \u02C8e\u026A s", "role": "o" }, { "id": "corp", "phones": "b \u02C8\xE6 s", "role": "o" }] }, "blessed": { "model": { "bias": 3.8304, "kind": "noun-verb", "weights": { "tn1=CAP": -1.5734, "tn1=CONJ": 1.8327, "tn1=PREP": 3.4327, "tn1n2=CAP+CAP": -0.5754, "tn2=S": -0.3107, "tp1=BE": 0.335, "w=the": -0.4856 } }, "senses": [{ "id": "adj", "phones": "b l \u02C8\u025B s \u026A d", "role": "n" }, { "id": "vrb", "phones": "b l \u02C8\u025B s t", "role": "v" }] }, "bologna": { "model": { "bias": [0, 0], "kind": "senses", "weights": [{}, { "cap": -4.4773 }] }, "senses": [{ "id": "geo", "phones": "b \u0259 l \u02C8o\u028A n j \u0259", "role": "o" }, { "id": "bologna", "phones": "b \u0259 l \u02C8o\u028A n i", "role": "o" }] }, "bow": { "model": { "bias": [0, 0], "kind": "senses", "weights": [{}, { "cap": -1.1189, "n1=.": 0.1451, "n2=</s>": 0.166, "n2=stern": 0.8309, "n2=the": -0.1985, "p1=a": -0.8012, "p1=bulbous": 1e-4, "p1=the": 0.9125, "p2=,": -1.3116, "p2=in": 0.0408, "p2=of": -0.9055, "tn1=PUNCT": -0.832, "tn1=W": 1.1978, "tn1n2=PUNCT+</s>": 0.1795, "tn2=</s>": 0.3013, "tn2=ING": 0.7026, "tn2=PREP": -0.0871, "tn2=W": -0.0161, "tp1n1=DET+W": 0.0476, "tp1n1=W+PREP": 1.1761, "tp2p1=S+PREP": 0.1517, "tp2p1=W+DET": 1.1324, "w=a": 0.1062, "w=and": -0.1183, "w=arrow": -0.5654, "w=began": -0.145, "w=bulbous": 0.026, "w=by": -0.1991, "w=her": 1.2871, "w=stern": 0.1539, "w=to": 0.1322, "w=two": 0.171, "w=with": -0.5472 }] }, "senses": [{ "id": "nou-knot", "phones": "b \u02C8o\u028A", "role": "o" }, { "id": "nou-ship", "phones": "b \u02C8a\u028A", "role": "o" }] }, "buffet": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "cap": 1.535, "tn1=PUNCT": 2.016 } }, "senses": [{ "id": "nou", "phones": "b \u0259 f \u02C8e\u026A", "role": "n" }, { "id": "vrb", "phones": "b \u02C8\u028C f \u026A t", "role": "v" }] }, "celtic": { "model": { "bias": [0, 0], "kind": "senses", "weights": [{}, { 'n1="': -0.3944, "p1=for": -0.4006, "tn1=CAP": 1.3551, "tn1=PREP": -1.1369, "tn1=PUNCT": -0.092, "tn1=S": 1.0933, "tn1=W": 0.679, "tn1n2=PUNCT+</s>": -0.4472, "tn1n2=PUNCT+PUNCT": -0.3461, "tn1n2=S+W": -0.9009, "tn2=CONJ": 0.3047, "tn2=PUNCT": 0.8833, "tp1=DET": 0.2659, "tp1=PREP": -0.3465, "tp1n1=DET+W": 0.9212, "tp2=CAP": -0.6507, "tp2=DET": 0.3479, "w=as": -0.7458, "w=at": -0.2145, "w=his": -0.8049, "w=league": -0.9908, "w=of": 1.6415, "w=with": -0.1941 }] }, "senses": [{ "id": "adj-nou-sports", "phones": "s \u02C8\u025B l t \u026A k", "role": "o" }, { "id": "celtic", "phones": "k \u02C8\u025B l t \u026A k", "role": "o" }] }, "close": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "n1=.": 0.1493, "n1=to": -28e-4, "n2=</s>": 0.3607, "p1=the": 0.6636, "p2=at": 0.2526, "tn1=PREP": 0.2561, "tn1=TO": -0.5417, "tn1=W": -0.1547, "tn1n2=PREP+DET": 0.1611, "tn1n2=PUNCT+</s>": 0.1836, "tn2=</s>": 0.4755, "tp1=DET": 2.7533, "w=at": 3.6621 } }, "senses": [{ "id": "adj-nou", "phones": "k l \u02C8o\u028A s", "role": "n" }, { "id": "vrb", "phones": "k l \u02C8o\u028A z", "role": "v" }] }, "combine": { "model": { "bias": 4.8159, "kind": "noun-verb", "weights": { "tp1=DET": 0.5897 } }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 m b a\u026A n", "role": "n" }, { "id": "vrb", "phones": "k \u0259 m b \u02C8a\u026A n", "role": "v" }] }, "compact": { "model": { "bias": -1.4258, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "adj-nou", "phones": "k \u02C8\u0251\u02D0 m p \xE6 k t", "role": "n" }, { "id": "vrb", "phones": "k \u0259 m p \u02C8\xE6 k t", "role": "v" }] }, "compound": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn1=DET": 0.307 } }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 m p a\u028A n d", "role": "n" }, { "id": "vrb", "phones": "k \u0259 m p \u02C8a\u028A n d", "role": "v" }] }, "compress": { "model": { "bias": [0, 1.7188], "kind": "senses", "weights": [{}, { "cap": -0.7845, "p1=to": 0.1723, "tn1=DET": 0.6114, "tn2=W": 1.2931, "tp1=PUNCT": -0.1787, "tp1=TO": 0.9235, "w=and": 0.5191, "w=in": -0.3431 }] }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 m p \u0279 \u025B s", "role": "n" }, { "id": "compress", "phones": "k \u0259 m p \u0279 \u02C8\u025B s", "role": "o" }] }, "conduct": { "model": { "bias": 0.887, "kind": "noun-verb", "weights": { "n2=,": 0.9176, "tn1=W": 2.0793, "tn2=PUNCT": 0.1754, "w=and": 0.4426, "w=to": 1.2826 } }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 n d \u0259 k t", "role": "n" }, { "id": "vrb", "phones": "k \u0259 n d \u02C8\u028C k t", "role": "v" }] }, "confines": { "model": { "bias": 0.7004, "kind": "noun-verb", "weights": { "p2=<s>": 0.2753, "tp2=<s>": 0.6297, "tp2p1=<s>+CAP": 0.6949, "w=a": 1.3904 } }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 n f a\u026A n z", "role": "n" }, { "id": "vrb", "phones": "k \u0259 n f \u02C8a\u026A n z", "role": "v" }] }, "conflict": { "model": { "bias": 0, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 n f l \u026A k t", "role": "n" }, { "id": "vrb", "phones": "k \u0259 n f l \u02C8\u026A k t", "role": "v" }] }, "conglomerate": { "model": { "bias": -1.3141, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "adj-nou", "phones": "k \u0259 \u014B \u0261 l \u02C8\u0251\u02D0 m \u025A \u0279 \u0259 t", "role": "n" }, { "id": "vrb", "phones": "k \u0259 \u014B \u0261 l \u02C8\u0251\u02D0 m \u025A \u0279 \u02CCe\u026A t", "role": "v" }] }, "conjugate": { "model": { "bias": -0.3299, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "adj-nou", "phones": "k \u02C8\u0251\u02D0 n d\u0292 \u0259 \u0261 \u0259 t", "role": "n" }, { "id": "vrb", "phones": "k \u02C8\u0251\u02D0 n d\u0292 \u0259 \u0261 \u02CCe\u026A t", "role": "v" }] }, "conscript": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn2=PUNCT": -0.1715 } }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 n s k \u0279 \u026A p t", "role": "n" }, { "id": "vrb", "phones": "k \u0259 n s k \u0279 \u02C8\u026A p t", "role": "v" }] }, "console": { "model": { "bias": -0.3017, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 n s o\u028A l", "role": "n" }, { "id": "vrb", "phones": "k \u0259 n s \u02C8o\u028A l", "role": "v" }] }, "consort": { "model": { "bias": -1.0312, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 n s \u0254\u02D0\u0279 t", "role": "n" }, { "id": "vrb", "phones": "k \u0259 n s \u02C8\u0254\u02D0\u0279 t", "role": "v" }] }, "construct": { "model": { "bias": 2.9231, "kind": "noun-verb", "weights": { "tn1=S": 0.4748 } }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 n s t \u0279 \u0259 k t", "role": "n" }, { "id": "vrb", "phones": "k \u0259 n s t \u0279 \u02C8\u028C k t", "role": "v" }] }, "consummate": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "p2=and": 0.4225, "tp1=CAP": 0.8112, "tp2p1=CONJ+CAP": 0.5906, "w=that": 0.1432 } }, "senses": [{ "id": "adj", "phones": "k \u02C8\u0251\u02D0 n s \u0259 m \u0259 t", "role": "n" }, { "id": "vrb", "phones": "k \u02C8\u0251\u02D0 n s \u0259 m \u02CCe\u026A t", "role": "v" }] }, "content": { "model": { "bias": [0, 3.7839], "kind": "senses", "weights": [{}, { "tp1=W": 3e-4 }] }, "senses": [{ "id": "adj-nou-vrb", "phones": "k \u0259 n t \u02C8\u025B n t", "role": "o" }, { "id": "nou", "phones": "k \u02C8\u0251\u02D0 n t \u025B n t", "role": "n" }] }, "contest": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tp2=W": 0.3988, "w=to": 0.0215 } }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 n t \u025B s t", "role": "n" }, { "id": "vrb", "phones": "k \u0259 n t \u02C8\u025B s t", "role": "v" }] }, "contract": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "w=a": -0.5108, "w=and": 0.4768 } }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 n t \u0279 \xE6 k t", "role": "n" }, { "id": "vrb", "phones": "k \u0259 n t \u0279 \u02C8\xE6 k t", "role": "v" }] }, "contrast": { "model": { "bias": 0, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 n t \u0279 \xE6 s t", "role": "n" }, { "id": "vrb", "phones": "k \u0259 n t \u0279 \u02C8\xE6 s t", "role": "v" }] }, "converse": { "model": { "bias": 1.1591, "kind": "noun-verb", "weights": { "n1=.": 93e-4, "n2=</s>": 0.0138, "tn1=PREP": 1.8614, "tn1=PUNCT": 0.6872, "tn1n2=PUNCT+</s>": 0.0335, "tp1=W": 1.4599, "tp2=W": 0.9733, "w=and": 0.3024 } }, "senses": [{ "id": "adj-nou", "phones": "k \u02C8\u0251\u02D0 n v \u025A s", "role": "n" }, { "id": "vrb", "phones": "k \u0259 n v \u02C8\u025C\u02D0 s", "role": "v" }] }, "convert": { "model": { "bias": 2.8394, "kind": "noun-verb", "weights": { "tp1=CAP": -1.706, "tp2=S": 0.8561, "w=a": -0.235, "w=into": 1.0107, "w=to": 0.647 } }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 n v \u025A t", "role": "n" }, { "id": "vrb", "phones": "k \u0259 n v \u02C8\u025C\u02D0 t", "role": "v" }] }, "convict": { "model": { "bias": 0, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0251\u02D0 n v \u026A k t", "role": "n" }, { "id": "vrb", "phones": "k \u0259 n v \u02C8\u026A k t", "role": "v" }] }, "coordinate": { "model": { "bias": 1.2383, "kind": "noun-verb", "weights": { "tn1=PUNCT": 0.4302, "w=and": 1.7973, "w=helped": 0.5834, "w=to": 0.3208 } }, "senses": [{ "id": "adj-nou", "phones": "k o\u028A \u02C8\u0254\u02D0\u0279 d \u026A n \u0259 t", "role": "n" }, { "id": "vrb", "phones": "k o\u028A \u02C8\u0254\u02D0\u0279 d \u026A n \u02CCe\u026A t", "role": "v" }] }, "correlate": { "model": { "bias": [0, 0.847], "kind": "senses", "weights": [{}, { "n1=in": -1.5399, "n1=with": 0.9018, "p1=neural": -85e-4, "tn1=W": 0.5253, "tp1=W": -1.1368, "w=neural": -0.5861, "w=not": 0.2444, "w=to": 2.0844, "w=with": 1.7693 }] }, "senses": [{ "id": "nou", "phones": "k \u02C8\u0254\u02D0 \u0279 \u0259l \u0259 t", "role": "n" }, { "id": "nou-vrb", "phones": "k \u02C8\u0254\u02D0 \u0279 \u0259 l \u02CCe\u026A t", "role": "o" }] }, "decrease": { "model": { "bias": 1.8055, "kind": "noun-verb", "weights": { "n1=in": -0.0154, "tn1=W": 0.6988, "tp2=W": 0.2771, "w=and": 1.2818, "w=pressure": 0.5654 } }, "senses": [{ "id": "nou", "phones": "d \u02C8i\u02D0 k \u0279 i\u02D0 s", "role": "n" }, { "id": "vrb", "phones": "d \u1D7B k \u0279 \u02C8i\u02D0 s", "role": "v" }] }, "defect": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn1=PUNCT": 0.4175, "tp1=W": -0.4394, "w=to": 1.3903 } }, "senses": [{ "id": "nou", "phones": "d \u02C8i\u02D0 f \u025B k t", "role": "n" }, { "id": "vrb", "phones": "d \u1D7B f \u02C8\u025B k t", "role": "v" }] }, "degenerate": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tp1=S": 3.1339 } }, "senses": [{ "id": "adj-nou", "phones": "d \u1D7B d\u0292 \u02C8\u025B n \u025A \u0279 \u0259 t", "role": "n" }, { "id": "vrb", "phones": "d \u1D7B d\u0292 \u02C8\u025B n \u025A \u0279 \u02CCe\u026A t", "role": "v" }] }, "delegate": { "model": { "bias": 0, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "d \u02C8\u025B l \u0259 \u0261 \u0259 t", "role": "n" }, { "id": "vrb", "phones": "d \u02C8\u025B l \u0259 \u0261 \u02CCe\u026A t", "role": "v" }] }, "deliberate": { "model": { "bias": -0.8233, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "adj", "phones": "d \u1D7B l \u02C8\u026A b \u025A \u0279 \u0259 t", "role": "n" }, { "id": "vrb", "phones": "d \u1D7B l \u02C8\u026A b \u025A \u0279 \u02CCe\u026A t", "role": "v" }] }, "desert": { "model": { "bias": -2.3, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "d \u02C8\u025B z \u025A t", "role": "n" }, { "id": "vrb", "phones": "d \u1D7B z \u02C8\u025C\u02D0 t", "role": "v" }] }, "deviate": { "model": { "bias": 4.456, "kind": "noun-verb", "weights": { "tn1=PREP": 2.2926 } }, "senses": [{ "id": "nou", "phones": "d \u02C8i\u02D0 v \u026A \u0259 t", "role": "n" }, { "id": "vrb", "phones": "d \u02C8i\u02D0 v \u026A \u02CCe\u026A t", "role": "v" }] }, "diagnoses": { "model": { "bias": 0.5346, "kind": "noun-verb", "weights": { "tn2=PREP": 0.9903, "tp2=DET": 0.0485, "tp2=S": 0.0895, "tp2=W": -0.21 } }, "senses": [{ "id": "nou", "phones": "d a\u026A\u0259 \u0261 n \u02C8o\u028A s i z", "role": "n" }, { "id": "vrb", "phones": "d a\u026A\u0259 \u0261 n \u02C8o\u028A s \u1D7B z", "role": "v" }] }, "diffuse": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "n2=the": 0.8518, "tn1=PREP": 1.7933, "tn1=W": -0.7432, "w=its": -0.8957, "w=membranes": 0.8939, "w=to": 2.1772 } }, "senses": [{ "id": "adj", "phones": "d \u026A f j \u02C8u\u02D0 s", "role": "n" }, { "id": "vrb", "phones": "d \u026A f j \u02C8u\u02D0 z", "role": "v" }] }, "discard": { "model": { "bias": 3.2689, "kind": "noun-verb", "weights": { "tp1=PUNCT": 1.6935 } }, "senses": [{ "id": "nou", "phones": "d \u02C8\u026A s k \u0251\u02D0\u0279 d", "role": "n" }, { "id": "vrb", "phones": "d \u026A s k \u02C8\u0251\u02D0\u0279 d", "role": "v" }] }, "discharge": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "p1=and": 0.0457, "w=and": 0.6339, "w=sea": 1.0031, "w=the": 0.647, "w=to": 0.5898 } }, "senses": [{ "id": "nou", "phones": "d \u02C8\u026A s t\u0283 \u0251\u02D0\u0279 d\u0292", "role": "n" }, { "id": "vrb", "phones": "d \u026A s t\u0283 \u02C8\u0251\u02D0\u0279 d\u0292", "role": "v" }] }, "discount": { "model": { "bias": -1.0279, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "d \u02C8\u026A s k a\u028A n t", "role": "n" }, { "id": "vrb", "phones": "d \u026A s k \u02C8a\u028A n t", "role": "v" }] }, "document": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn1=S": 0.2065, "w=to": 1.7204 } }, "senses": [{ "id": "nou", "phones": "d \u02C8\u0251\u02D0 k j \u028A m \u0259 n t", "role": "n" }, { "id": "vrb", "phones": "d \u02C8\u0251\u02D0 k j \u028A m \u02CC\u025B n t", "role": "v" }] }, "dove": { "model": { "bias": [0, -1.6196], "kind": "senses", "weights": [{}, { "tn1=PREP": 1.9864, "tn1=PUNCT": -0.2724, "tn1n2=PREP+DET": 0.3933, "tp1=DET": -0.847, "tp1=PUNCT": -0.1945, "w=the": -0.7337 }] }, "senses": [{ "id": "dove", "phones": "d \u02C8\u028C v", "role": "o" }, { "id": "vrb", "phones": "d \u02C8o\u028A v", "role": "v" }] }, "duplicate": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn2=S": -0.369, "w=and": -0.4913, "w=the": -0.5299 } }, "senses": [{ "id": "adj-nou", "phones": "d \u02C8u\u02D0 p l \u026A k \u0259 t", "role": "n" }, { "id": "vrb", "phones": "d \u02C8u\u02D0 p l \u026A k \u02CCe\u026A t", "role": "v" }] }, "elaborate": { "model": { "bias": -1.7982, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "adj", "phones": "\u026A l \u02C8\xE6 b \u025A \u0279 \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u026A l \u02C8\xE6 b \u025A \u0279 \u02CCe\u026A t", "role": "v" }] }, "entrance": { "model": { "bias": -0.0253, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "\u02C8\u025B n t \u0279 \u0259 n s", "role": "n" }, { "id": "vrb", "phones": "\u025B n t \u0279 \u02C8\xE6 n s", "role": "v" }] }, "escort": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "w=to": 0.2512 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u025B s k \u0254\u02D0\u0279 t", "role": "n" }, { "id": "vrb", "phones": "\u025B s k \u02C8\u0254\u02D0\u0279 t", "role": "v" }] }, "estimate": { "model": { "bias": 0.8174, "kind": "noun-verb", "weights": { "n1=of": -0.0954, "w=and": 0.9096, "w=million": 0.0228 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u025B s t \u026A m \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u02C8\u025B s t \u026A m \u02CCe\u026A t", "role": "v" }] }, "excuse": { "model": { "bias": 0, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "\u025B k s k j \u02C8u\u02D0 s", "role": "n" }, { "id": "vrb", "phones": "\u025B k s k j \u02C8u\u02D0 z", "role": "v" }] }, "expatriate": { "model": { "bias": -0.8968, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "\u025B k s p \u02C8e\u026A t \u0279 \u026A \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u025B k s p \u02C8e\u026A t \u0279 \u026A \u02CCe\u026A t", "role": "v" }] }, "exploit": { "model": { "bias": 2.292, "kind": "noun-verb", "weights": { "tn1=W": 1.4139 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u025B k s p l \u0254\u026A t", "role": "n" }, { "id": "vrb", "phones": "\u025B k s p l \u02C8\u0254\u026A t", "role": "v" }] }, "export": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn2=CAP": 1.1816, "w=to": 1.5427 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u025B k s p o\u02D0\u0279 t", "role": "n" }, { "id": "vrb", "phones": "\u025B k s p \u02C8o\u02D0\u0279 t", "role": "v" }] }, "expose": { "model": { "bias": 4.8197, "kind": "noun-verb", "weights": { "tn1=S": 1.2553 } }, "senses": [{ "id": "nou", "phones": "\u02CC\u025B k s p o\u028A z \u02C8e\u026A", "role": "n" }, { "id": "vrb", "phones": "\u025B k s p \u02C8o\u028A z", "role": "v" }] }, "extract": { "model": { "bias": 1.2617, "kind": "noun-verb", "weights": { "tn1=S": 0.8543, "w=to": 0.3602 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u025B k s t \u0279 \xE6 k t", "role": "n" }, { "id": "vrb", "phones": "\u025B k s t \u0279 \u02C8\xE6 k t", "role": "v" }] }, "fragment": { "model": { "bias": 0, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "f \u0279 \u02C8\xE6 \u0261 m \u0259 n t", "role": "n" }, { "id": "vrb", "phones": "f \u0279 \xE6 \u0261 m \u02C8\u025B n t", "role": "v" }] }, "frequent": { "model": { "bias": -1.3123, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "adj", "phones": "f \u0279 \u02C8i\u02D0 k w \u0259 n t", "role": "n" }, { "id": "vrb", "phones": "f \u0279 \u026A k w \u02C8\u025B n t", "role": "v" }] }, "graduate": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn1=W": -1.4364, "tn2=PREP": -0.0919, "w=and": -0.2704 } }, "senses": [{ "id": "adj-nou", "phones": "\u0261 \u0279 \u02C8\xE6 d\u0292 u\u02D0 \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u0261 \u0279 \u02C8\xE6 d\u0292 u\u02D0 \u02CCe\u026A t", "role": "v" }] }, "house": { "model": { "bias": 0, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "h \u02C8a\u028A s", "role": "n" }, { "id": "vrb", "phones": "h \u02C8a\u028A z", "role": "v" }] }, "impact": { "model": { "bias": -0.4225, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "\u02C8\u026A m p \xE6 k t", "role": "n" }, { "id": "vrb", "phones": "\u026A m p \u02C8\xE6 k t", "role": "v" }] }, "implant": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "w=others": 0.3373 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u026A m p l \xE6 n t", "role": "n" }, { "id": "vrb", "phones": "\u026A m p l \u02C8\xE6 n t", "role": "v" }] }, "implement": { "model": { "bias": 2.4252, "kind": "noun-verb", "weights": { "tn1=W": 0.2051, "w=and": 0.5111 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u026A m p l \u0259 m \u0259 n t", "role": "n" }, { "id": "vrb", "phones": "\u02C8\u026A m p l \u0259 m \u02CC\u025B n t", "role": "v" }] }, "import": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "w=from": 0.0728, "w=to": 0.8431 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u026A m p o\u02D0\u0279 t", "role": "n" }, { "id": "vrb", "phones": "\u026A m p \u02C8o\u02D0\u0279 t", "role": "v" }] }, "incense": { "model": { "bias": -2.2693, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "\u02C8\u026A n s \u025B n s", "role": "n" }, { "id": "vrb", "phones": "\u026A n s \u02C8\u025B n s", "role": "v" }] }, "incline": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn1n2=TO+W": 1.5038, "tp1=W": 1.2796 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u026A \u014B k l a\u026A n", "role": "n" }, { "id": "vrb", "phones": "\u026A \u014B k l \u02C8a\u026A n", "role": "v" }] }, "increase": { "model": { "bias": 1.672, "kind": "noun-verb", "weights": { "tn1=PREP": -0.3878, "tn1=W": 1.3402, "tn2=W": 0.5638, "tp2=W": 0.7385, "w=and": 0.1276 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u026A \u014B k \u0279 i\u02D0 s", "role": "n" }, { "id": "vrb", "phones": "\u026A \u014B k \u0279 \u02C8i\u02D0 s", "role": "v" }] }, "increment": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "p1=then": 0.8303, "tp1=ADV": 0.3399, "tp2p1=S+S": 0.6282, "w=then": 0.6473 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u026A \u014B k \u0279 \u0259 m \u0259 n t", "role": "n" }, { "id": "vrb", "phones": "\u02C8\u026A \u014B k \u0279 \u0259 m \u02CC\u025B n t", "role": "v" }] }, "initiate": { "model": { "bias": 3.3647, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "\u026A n \u02C8\u026A \u0283 \u026A \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u026A n \u02C8\u026A \u0283 \u026A \u02CCe\u026A t", "role": "v" }] }, "insert": { "model": { "bias": 2.6477, "kind": "noun-verb", "weights": { "cap": 3.3561, "n1=,": 1.7937, "p1=,": 0.131, "tp1=CAP": -0.6906, "tp1=PUNCT": 0.1015, "w=a": -0.9248, "w=and": 0.2643, "w=are": 0.3226, "w=is": -0.8648, "w=or": 0.9148 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u026A n s \u025C\u02D0 t", "role": "n" }, { "id": "vrb", "phones": "\u026A n s \u02C8\u025C\u02D0 t", "role": "v" }] }, "instrument": { "model": { "bias": -1.5123, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "\u02C8\u026A n s t \u0279 \u0259 m \u0259 n t", "role": "n" }, { "id": "vrb", "phones": "\u02C8\u026A n s t \u0279 \u0259 m \u02CC\u025B n t", "role": "v" }] }, "insult": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn1=CAP": 0.7639, "w=to": 0.245 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u026A n s \u028C l t", "role": "n" }, { "id": "vrb", "phones": "\u026A n s \u02C8\u028C l t", "role": "v" }] }, "interchange": { "model": { "bias": -0.1171, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "\u02C8\u026A n t \u025A t\u0283 \u02CCe\u026A n d\u0292", "role": "n" }, { "id": "vrb", "phones": "\u02CC\u026A n t \u025A t\u0283 \u02C8e\u026A n d\u0292", "role": "v" }] }, "intimate": { "model": { "bias": -1.2482, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "adj", "phones": "\u02C8\u026A n t \u026A m \u0259 t", "role": "n" }, { "id": "vrb", "phones": "\u02C8\u026A n t \u026A m \u02CCe\u026A t", "role": "v" }] }, "intrigue": { "model": { "bias": [0, -2.2022], "kind": "senses", "weights": [{}, { "p1=to": 0.4253, "tn1=PUNCT": -1.3016, "tn2=PUNCT": 1.7017, "tp1=TO": 1.4468, "tp1=W": -0.3776, "tp2=DET": 0.1606, "w=of": -1.0977 }] }, "senses": [{ "id": "nou", "phones": "\u02C8\u026A n t \u0279 \u026A \u0261", "role": "n" }, { "id": "nou-vrb", "phones": "\u026A n t \u0279 \u02C8i\u02D0 \u0261", "role": "o" }] }, "invalid": { "model": { "bias": [0, -2.2695], "kind": "senses", "weights": [{}, { "cap": 0.5383, "n2=the": 0.227, "p2=a": 1.4097, "tn2=W": -0.6494, "tp1=BE": -0.5924, "tp1=DET": 2.9202, "tp1=POSS": 0.0335, "tp1n1=DET+PREP": 0.5838, "tp1n1=DET+PUNCT": 0.9869, "tp1n1=DET+W": -1.4506, "tp1n1=POSS+W": 0.7722, "tp2=W": 0.226, "tp2p1=W+POSS": 0.695, "w=he": 0.0229, "w=her": 0.203, "w=to": 0.5427 }] }, "senses": [{ "id": "adj", "phones": "\u026A n v \u02C8\xE6 l \u026A d", "role": "n" }, { "id": "nou", "phones": "\u02C8\u026A n v \u0259l \u026A d", "role": "n" }] }, "invert": { "model": { "bias": 2.5922, "kind": "noun-verb", "weights": { "cap": 0.5552, "n1=sugar": -0.9297, "tp1=PUNCT": 0.8629, "tp2=W": 0.5131, "w=a": -0.7931, "w=as": 0.8548, "w=of": 1.6477, "w=sugar": -0.5558 } }, "senses": [{ "id": "adj-nou", "phones": "\u02C8\u026A n v \u025A t", "role": "n" }, { "id": "vrb", "phones": "\u026A n v \u02C8\u025C\u02D0 t", "role": "v" }] }, "invite": { "model": { "bias": 2.2544, "kind": "noun-verb", "weights": { "tn1=W": 0.8134, "tn2=S": 0.2035, "tn2=W": -0.0514, "w=and": 0.2557 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u026A n v a\u026A t", "role": "n" }, { "id": "vrb", "phones": "\u026A n v \u02C8a\u026A t", "role": "v" }] }, "isolate": { "model": { "bias": [0, 0], "kind": "senses", "weights": [{}, { "p1=to": 0.9665, "p2=a": -0.0169, "tn1=DET": 1.3071, "tn1=PREP": -0.1644, "tn1=S": 0.3905, "tn2=S": 0.393, "tn2=W": 0.4537, "tp1=TO": 0.0843, "tp1=W": -2.5788, "tp2=DET": -1.1791, "tp2=W": 0.5774, "w=from": 0.2089, "w=in": 0.3103, "w=they": 0.4931, "w=to": 2.0333, "w=with": -0.0693 }] }, "senses": [{ "id": "nou", "phones": "\u02C8a\u026A s \u0259l \u0259 t", "role": "n" }, { "id": "isolate", "phones": "\u02C8a\u026A s \u0259 l \u02CCe\u026A t", "role": "o" }] }, "jesus": { "model": { "bias": [0, -3.7371], "kind": "senses", "weights": [{}, { "tp1n1=W+CAP": 2.0292, "w=for": 1.3986, "w=of": -0.1475 }] }, "senses": [{ "id": "jesus", "phones": "d\u0292 \u02C8i\u02D0 z \u0259 s", "role": "o" }, { "id": "es", "phones": "h e\u026A s \u02C8u\u02D0 s", "role": "o" }] }, "job": { "model": { "bias": [0, 1e6], "kind": "senses", "weights": [{}, {}] }, "senses": [{ "id": "bible", "phones": "d\u0292 \u02C8o\u028A b", "role": "o" }, { "id": "job", "phones": "d\u0292 \u02C8\u0251\u02D0 b", "role": "o" }] }, "laminate": { "model": { "bias": -0.3864, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "l \u02C8\xE6 m \u026A n \u0259 t", "role": "n" }, { "id": "vrb", "phones": "l \u02C8\xE6 m \u026A n \u02CCe\u026A t", "role": "v" }] }, "lead": { "model": { "bias": [0, 1.2397], "kind": "senses", "weights": [{}, { "first": -0.0302, "n1=roles": 0.0475, "n1=to": 1.244, "p1=<s>": -0.0211, "p1=the": 1.5303, "tn1=BE": -3.0491, "tn1=DET": 0.4039, "tn1=ED": -13e-4, "tn1=TO": 0.0953, "tn1=W": 0.7875, "tn2=PUNCT": 0.8273, "tn2=W": 0.1367, "tp1=<s>": -0.1429, "tp1=PREP": -0.5247, "tp1n1=CAP+PUNCT": -1.6296, "tp2=DET": 0.0248, "tp2=PREP": 0.2708, "tp2p1=<s>+<s>": -0.0138, "w=of": -1.4478, "w=red": -0.3899, "w=roles": 0.0134, "w=the": 0.2734, "w=to": 0.5064, "w=were": -0.2525 }] }, "senses": [{ "id": "nou", "phones": "l \u02C8\u025B d", "role": "n" }, { "id": "nou-vrb", "phones": "l \u02C8i\u02D0 d", "role": "o" }] }, "learned": { "model": { "bias": 5.3875, "kind": "noun-verb", "weights": { "tn1=S": -0.0131, "tn1=W": 0.237, "tn1n2=S+PREP": -0.0747, "tp1=DET": 1.5177, "w=and": -0.6358, "w=it": 0.235, "w=the": 0.558 } }, "senses": [{ "id": "adj", "phones": "l \u02C8\u025C\u02D0 n \u026A d", "role": "n" }, { "id": "vrb", "phones": "l \u02C8\u025C\u02D0 n d", "role": "v" }] }, "live": { "model": { "bias": 0.5332, "kind": "noun-verb", "weights": { "n1=in": 2.1724, "tn1=LY": 0.0986, "tn1=PREP": 0.4525, "tn1=W": -0.3904, "tn2=PREP": -0.2527, "tn2=W": 0.0514, "tp2=CAP": 0.1566, "w=in": 1.8919, "w=two": 0.9876 } }, "senses": [{ "id": "adj", "phones": "l \u02C8a\u026A v", "role": "n" }, { "id": "vrb", "phones": "l \u02C8\u026A v", "role": "v" }] }, "lives": { "model": { "bias": 2.4926, "kind": "noun-verb", "weights": { "n1=in": 1.099, "n1=on": 0.3569, "p2=<s>": 0.1897, "tn1=PREP": 0.9524, "tp2=<s>": 0.6529, "tp2=DET": 2.5636, "tp2=S": -0.5611, "w=in": 0.6375, "w=of": -0.5849, "w=sisters": -0.4628, "w=their": -0.6091, "w=where": 0.4337, "w=with": 0.3457 } }, "senses": [{ "id": "nou", "phones": "l \u02C8a\u026A v z", "role": "n" }, { "id": "vrb", "phones": "l \u02C8\u026A v z", "role": "v" }] }, "mate": { "model": { "bias": [0, 3.7842], "kind": "senses", "weights": [{}, {}] }, "senses": [{ "id": "nou", "phones": "m \u02C8\u0251\u02D0 t e\u026A", "role": "n" }, { "id": "mate", "phones": "m \u02C8e\u026A t", "role": "o" }] }, "minute": { "model": { "bias": [0, 2.3994], "kind": "senses", "weights": [{}, { "tn1=W": -0.7621, "tp1=PUNCT": 2.0219, "tp1=W": 0.9849, "tp1n1=DET+W": -0.9854, "tp1n1=PREP+W": -2.6316, "w=is": -1.3055, "w=the": 11e-4 }] }, "senses": [{ "id": "adj", "phones": "m a\u026A n \u02C8u\u02D0 t", "role": "n" }, { "id": "minute", "phones": "m \u02C8\u026A n \u026A t", "role": "o" }] }, "misuse": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn1=PREP": -0.7173, "tn1=W": 0.7709, "w=of": -0.5362 } }, "senses": [{ "id": "nou", "phones": "m \u026A s j \u02C8u\u02D0 s", "role": "n" }, { "id": "vrb", "phones": "m \u026A s j \u02C8u\u02D0 z", "role": "v" }] }, "mobile": { "model": { "bias": [3.3899, -3.3899, -1e6], "kind": "senses", "weights": [{ "cap": -0.8729, "tn1=PUNCT": -0.228, "tn1=W": 0.2457, "tp1=PREP": -0.0274, "tp2=CAP": -1.0705, "w=of": -0.0162, "w=the": 59e-4 }, { "cap": 0.8729, "tn1=PUNCT": 0.228, "tn1=W": -0.2457, "tp1=PREP": 0.0274, "tp2=CAP": 1.0705, "w=of": 0.0162, "w=the": -59e-4 }, {}] }, "senses": [{ "id": "mobile", "phones": "m \u02C8o\u028A b \u0259l", "role": "o" }, { "id": "geo", "phones": "m o\u028A b \u02C8i\u02D0 l", "role": "o" }, { "id": "nou-art", "phones": "m \u02C8o\u028A b i\u02D0 l", "role": "o" }] }, "moderate": { "model": { "bias": -2.0892, "kind": "noun-verb", "weights": { "tp2=W": -1.4834 } }, "senses": [{ "id": "adj-nou", "phones": "m \u02C8\u0251\u02D0 d \u025A \u0279 \u0259 t", "role": "n" }, { "id": "vrb", "phones": "m \u02C8\u0251\u02D0 d \u025A \u0279 \u02CCe\u026A t", "role": "v" }] }, "moped": { "model": { "bias": -0.6152, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "m \u02C8o\u028A p \u025B d", "role": "n" }, { "id": "vrb", "phones": "m \u02C8o\u028A p t", "role": "v" }] }, "mouth": { "model": { "bias": 0, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "m \u02C8a\u028A \u03B8", "role": "n" }, { "id": "vrb", "phones": "m \u02C8a\u028A \xF0", "role": "v" }] }, "nestle": { "model": { "bias": [0, 0.821], "kind": "senses", "weights": [{}, { "cap": -4.8328, "p2=<s>": -0.4082, "tn1=PREP": 83e-4, "tn1n2=PREP+DET": 0.0343, "tp1=S": 0.1924, "tp2=<s>": -2.2373, "w=and": 0.1986 }] }, "senses": [{ "id": "nam", "phones": "n \u02C8\u025B s l i", "role": "o" }, { "id": "vrb", "phones": "n \u02C8\u025B s \u0259l", "role": "v" }] }, "object": { "model": { "bias": -0.0958, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "\u02C8\u0251\u02D0 b d\u0292 \u025B k t", "role": "n" }, { "id": "vrb", "phones": "\u0259 b d\u0292 \u02C8\u025B k t", "role": "v" }] }, "ornament": { "model": { "bias": -0.605, "kind": "noun-verb", "weights": { "tp1=CONJ": -0.2638 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u0254\u02D0\u0279 n \u0259 m \u0259 n t", "role": "n" }, { "id": "vrb", "phones": "\u02C8\u0254\u02D0\u0279 n \u0259 m \u02CC\u025B n t", "role": "v" }] }, "overthrow": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "w=and": 0.0392, "w=to": 2.3046 } }, "senses": [{ "id": "nou", "phones": "\u02C8o\u028A v \u025A \u03B8 \u0279 \u02CCo\u028A", "role": "n" }, { "id": "vrb", "phones": "\u02CCo\u028A v \u025A \u03B8 \u0279 \u02C8o\u028A", "role": "v" }] }, "pasty": { "model": { "bias": [0, 0.265], "kind": "senses", "weights": [{}, { "cap": 0.0961, "n2=.": -0.8336, "p1=the": 1.07, "tn1=W": -1.2005, "tn2=W": -0.731, "tp1=CAP": 2.3366, "tp1=DET": 1.4056, "tp1=PUNCT": 0.0692, "tp2=BE": -0.2509, "tp2=DET": 1.7276, "w=is": -0.1915, "w=products": -0.0998, "w=the": 0.401 }] }, "senses": [{ "id": "adj", "phones": "p \u02C8e\u026A s t i", "role": "n" }, { "id": "nou", "phones": "p \u02C8\xE6 s t i", "role": "n" }] }, "perfect": { "model": { "bias": -0.791, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "adj", "phones": "p \u02C8\u025C\u02D0 f \u025B k t", "role": "n" }, { "id": "vrb", "phones": "p \u025A f \u02C8\u025B k t", "role": "v" }] }, "perfume": { "model": { "bias": -0.1505, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "p \u02C8\u025C\u02D0 f j u\u02D0 m", "role": "n" }, { "id": "vrb", "phones": "p \u025A f j \u02C8u\u02D0 m", "role": "v" }] }, "permit": { "model": { "bias": 1.6055, "kind": "noun-verb", "weights": { "tp1=S": 0.9875, "w=of": 0.1926 } }, "senses": [{ "id": "nou", "phones": "p \u02C8\u025C\u02D0 m \u026A t", "role": "n" }, { "id": "vrb", "phones": "p \u025A m \u02C8\u026A t", "role": "v" }] }, "pervert": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "w=the": 0.8871 } }, "senses": [{ "id": "nou", "phones": "p \u02C8\u025C\u02D0 v \u025A t", "role": "n" }, { "id": "vrb", "phones": "p \u025A v \u02C8\u025C\u02D0 t", "role": "v" }] }, "pigment": { "model": { "bias": -0.8014, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "p \u02C8\u026A \u0261 m \u0259 n t", "role": "n" }, { "id": "vrb", "phones": "p \u026A \u0261 m \u02C8\u025B n t", "role": "v" }] }, "polish": { "model": { "bias": [0, 0], "kind": "senses", "weights": [{}, { "cap": -4.2652, "p2=<s>": -0.3353, "tn1=W": -0.3047, "tp2=<s>": -0.2069 }] }, "senses": [{ "id": "geo", "phones": "p \u02C8o\u028A l \u026A \u0283", "role": "o" }, { "id": "polish", "phones": "p \u02C8\u0251\u02D0 l \u026A \u0283", "role": "o" }] }, "postulate": { "model": { "bias": 0.5313, "kind": "noun-verb", "weights": { "tp1=S": 0.8861 } }, "senses": [{ "id": "nou", "phones": "p \u02C8\u0251\u02D0 s t\u0283 \u0259l \u0259 t", "role": "n" }, { "id": "vrb", "phones": "p \u02C8\u0251\u02D0 s t\u0283 \u0259 l \u02CCe\u026A t", "role": "v" }] }, "precipitate": { "model": { "bias": 1.6847, "kind": "noun-verb", "weights": { "tn1=W": 1.4355, "w=a": -0.5385, "w=and": 1.4685, "w=barium": 0.3849, "w=that": -1.2313 } }, "senses": [{ "id": "adj-nou", "phones": "p \u0279 \u026A s \u02C8\u026A p \u026A \u027E \u0259 t", "role": "n" }, { "id": "vrb", "phones": "p \u0279 \u026A s \u02C8\u026A p \u026A t \u02CCe\u026A t", "role": "v" }] }, "predicate": { "model": { "bias": -0.5296, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "p \u0279 \u02C8\u025B d \u026A k \u0259 t", "role": "n" }, { "id": "vrb", "phones": "p \u0279 \u02C8\u025B d \u026A k \u02CCe\u026A t", "role": "v" }] }, "present": { "model": { "bias": -0.3451, "kind": "noun-verb", "weights": { "w=in": -0.7651, "w=with": 0.9538 } }, "senses": [{ "id": "adj-nou", "phones": "p \u0279 \u02C8\u025B z \u0259 n t", "role": "n" }, { "id": "vrb", "phones": "p \u0279 \u1D7B z \u02C8\u025B n t", "role": "v" }] }, "produce": { "model": { "bias": 3.2498, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "p \u0279 \u02C8o\u028A d u\u02D0 s", "role": "n" }, { "id": "vrb", "phones": "p \u0279 \u0259 d \u02C8u\u02D0 s", "role": "v" }] }, "progress": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "n2=the": 0.6779, "tn1=PREP": -1.0081, "tn1=PUNCT": 0.2195, "tn2=DET": 1.6529, "tp1=S": 0.4983, "tp1n1=S+PUNCT": 2.3993, "w=as": 0.4898 } }, "senses": [{ "id": "nou", "phones": "p \u0279 \u02C8\u0251\u02D0 \u0261 \u0279 \u025B s", "role": "n" }, { "id": "vrb", "phones": "p \u0279 \u0259 \u0261 \u0279 \u02C8\u025B s", "role": "v" }] }, "project": { "model": { "bias": 0, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "p \u0279 \u02C8\u0251\u02D0 d\u0292 \u025B k t", "role": "n" }, { "id": "vrb", "phones": "p \u0279 \u0259 d\u0292 \u02C8\u025B k t", "role": "v" }] }, "protest": { "model": { "bias": 0, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "p \u0279 \u02C8o\u028A t \u025B s t", "role": "n" }, { "id": "vrb", "phones": "p \u0279 \u0259 t \u02C8\u025B s t", "role": "v" }] }, "ravel": { "model": { "bias": [0, -2.4847], "kind": "senses", "weights": [{}, { "cap": -1.846 }] }, "senses": [{ "id": "nam", "phones": "\u0279 \u0259 v \u02C8\u025B l", "role": "o" }, { "id": "nou", "phones": "\u0279 \u02C8\xE6 v \u0259l", "role": "n" }] }, "read": { "model": { "bias": [0, 0.3195], "kind": "senses", "weights": [{}, { "cap": 1.1231, "n1=the": 0.2502, "n2=the": -0.4753, "n2=write": 0.1929, "p1=also": -0.5193, "p1=he": -0.049, "p1=to": 0.6309, "p2=,": -1.888, "tn1=DET": 0.4086, "tn1=PREP": -0.6246, "tn2=ADV": 1.6577, "tn2=CAP": -0.5068, "tn2=S": 0.4909, "tp1=BE": -1.9933, "tp1=MODAL": 1.342, "tp1=SUBJ": -1.2926, "tp1=TO": 1.4452, "tp1=W": 0.0334, "tp1n1=PUNCT+W": -0.5571, "tp2=BE": -0.8439, "tp2=MODAL": 0.8244, "tp2=PREP": 0.1362, "tp2p1=DO+NEG": 0.087, "tp2p1=SUBJ+ADV": -0.6786, "w=a": 0.0215, "w=and": 0.0657, "w=as": 0.2383, "w=be": -0.4351, "w=by": -0.139, "w=he": -0.4189, "w=its": 0.5206, "w=on": -0.6018, "w=the": -0.151, "w=to": 0.1999, "w=write": 0.11 }] }, "senses": [{ "id": "past", "phones": "\u0279 \u02C8\u025B d", "role": "o" }, { "id": "present", "phones": "\u0279 \u02C8i\u02D0 d", "role": "o" }] }, "reading": { "model": { "bias": [0, -3.5824], "kind": "senses", "weights": [{}, { "cap": 6.2904 }] }, "senses": [{ "id": "en", "phones": "\u0279 \u02C8i\u02D0 d \u026A \u014B", "role": "o" }, { "id": "geo", "phones": "\u0279 \u02C8\u025B d \u026A \u014B", "role": "o" }] }, "rebel": { "model": { "bias": -0.0121, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "\u0279 \u02C8\u025B b \u0259l", "role": "n" }, { "id": "vrb", "phones": "\u0279 \u1D7B b \u02C8\u025B l", "role": "v" }] }, "record": { "model": { "bias": 0, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "\u0279 \u02C8\u025B k \u025A d", "role": "n" }, { "id": "vrb", "phones": "\u0279 \u1D7B k \u02C8\u0254\u02D0\u0279 d", "role": "v" }] }, "recount": { "model": { "bias": 1.4237, "kind": "noun-verb", "weights": { "tn1n2=PREP+DET": 0.3032, "tp2=PREP": 1.75, "w=from": 0.1584, "w=he": 0.0186, "w=of": 1.8627 } }, "senses": [{ "id": "nou", "phones": "\u0279 \u02C8i\u02D0 k a\u028A n t", "role": "n" }, { "id": "vrb", "phones": "\u0279 \u1D7B k \u02C8a\u028A n t", "role": "v" }] }, "refund": { "model": { "bias": 0, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "\u0279 \u02C8i\u02D0 f \u028C n d", "role": "n" }, { "id": "vrb", "phones": "\u0279 \u1D7B f \u02C8\u028C n d", "role": "v" }] }, "refuse": { "model": { "bias": 2.3718, "kind": "noun-verb", "weights": { "cap": 0.4878, "n1=to": 1.4828, "tn1=PUNCT": 0.7423, "tn1=TO": 1.5402, "tn1=W": -0.4851, "tn2=CAP": 0.4415, "tp1=PUNCT": 0.795, "w=the": 1.0147 } }, "senses": [{ "id": "nou", "phones": "\u0279 \u02C8\u025B f j u\u02D0 s", "role": "n" }, { "id": "vrb", "phones": "\u0279 \u1D7B f j \u02C8u\u02D0 z", "role": "v" }] }, "reject": { "model": { "bias": 3.7819, "kind": "noun-verb", "weights": { "tn2=CAP": 0.2139, "w=the": 0.2116 } }, "senses": [{ "id": "nou", "phones": "\u0279 \u02C8i\u02D0 d\u0292 \u025B k t", "role": "n" }, { "id": "vrb", "phones": "\u0279 \u1D7B d\u0292 \u02C8\u025B k t", "role": "v" }] }, "rerelease": { "model": { "bias": [0, -2.1165], "kind": "senses", "weights": [{}, { "p1=to": 1.9787, "tn1=PREP": -0.6945, "tn1=PUNCT": -0.1691, "tp1=DET": -0.2498, "tp1=TO": 1.2617, "w=a": -0.1369, "w=the": -1.1077, "w=to": 2.0792 }] }, "senses": [{ "id": "rerelease", "phones": "\u0279 \u02C8i\u0259 \u0279 \u0259 l \u02CCi\u02D0 s", "role": "o" }, { "id": "vrb", "phones": "\u0279 \u02CCi\u0259 \u0279 \u0259 l \u02C8i\u02D0 s", "role": "v" }] }, "resume": { "model": { "bias": 3.202, "kind": "noun-verb", "weights": { "tn1=ING": 0.2006 } }, "senses": [{ "id": "nou", "phones": "\u0279 \u02C8\u025B z \u0259 m \u02CCe\u026A", "role": "n" }, { "id": "vrb", "phones": "\u0279 \u1D7B z \u02C8u\u02D0 m", "role": "v" }] }, "retard": { "model": { "bias": 2.9004, "kind": "noun-verb", "weights": { "tn1=W": 1.9878, "tp1=PUNCT": -0.0618, "tp1=W": 1.1149, "tp2=W": 0.6665, "w=a": -0.6521 } }, "senses": [{ "id": "nou", "phones": "\u0279 \u02C8i\u02D0 t \u0251\u02D0\u0279 d", "role": "n" }, { "id": "vrb", "phones": "\u0279 \u1D7B t \u02C8\u0251\u02D0\u0279 d", "role": "v" }] }, "rodeo": { "model": { "bias": [0, 3.3466], "kind": "senses", "weights": [{}, { "n1=,": -0.4021, "p1=del": -1.4939, "tp2p1=CAP+W": -1.5572, "w=del": -0.0945 }] }, "senses": [{ "id": "geo", "phones": "\u0279 o\u028A d \u02C8e\u026A o\u028A", "role": "o" }, { "id": "rodeo", "phones": "\u0279 \u02C8o\u028A d \u026A \u02CCo\u028A", "role": "o" }] }, "row": { "model": { "bias": [0, -1e6], "kind": "senses", "weights": [{}, {}] }, "senses": [{ "id": "1", "phones": "\u0279 \u02C8o\u028A", "role": "o" }, { "id": "2", "phones": "\u0279 \u02C8a\u028A", "role": "o" }] }, "sake": { "model": { "bias": [0, -1.5781], "kind": "senses", "weights": [{}, { "p2=for": 2.086, "tp1=PREP": -0.1808, "w=are": -0.1788, "w=for": 3.697 }] }, "senses": [{ "id": "jp", "phones": "s \u02C8\u0251\u02D0 k e\u026A", "role": "o" }, { "id": "sake", "phones": "s \u02C8e\u026A k", "role": "o" }] }, "separate": { "model": { "bias": -0.1908, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "adj", "phones": "s \u02C8\u025B p \u025A \u0279 \u0259 t", "role": "n" }, { "id": "vrb", "phones": "s \u02C8\u025B p \u025A \u0279 \u02CCe\u026A t", "role": "v" }] }, "sow": { "model": { "bias": [0, 0], "kind": "senses", "weights": [{}, { "cap": -1.0132, "n1=,": 0.1666, "p1=to": 0.0266, "p2=the": -0.4073, "tn1=S": -0.1982, "tn1=W": 0.4926, "tn1n2=PUNCT+PUNCT": -1.267, "tn2=PREP": 0.2795, "tn2=S": 0.9062, "tn2=W": 0.3987, "tp1=CAP": -0.0118, "tp1=CONJ": 0.4021, "tp1=DET": -2.3168, "tp1=PUNCT": -0.3474, "tp1=SUBJ": 1.3059, "tp1=TO": 2.559, "tp1=W": -0.3511, "w=in": 0.4334, "w=plow": 0.0525, "w=reap": 1.1023 }] }, "senses": [{ "id": "nou", "phones": "s \u02C8a\u028A", "role": "n" }, { "id": "sow", "phones": "s \u02C8o\u028A", "role": "o" }] }, "subject": { "model": { "bias": -1.7854, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "adj-nou", "phones": "s \u02C8\u028C b d\u0292 \u025B k t", "role": "n" }, { "id": "vrb", "phones": "s \u0259 b d\u0292 \u02C8\u025B k t", "role": "v" }] }, "subordinate": { "model": { "bias": -0.3507, "kind": "noun-verb", "weights": { "tn1=S": -0.9328 } }, "senses": [{ "id": "adj-nou", "phones": "s \u0259 b \u02C8\u0254\u02D0\u0279 d \u026A n \u0259 t", "role": "n" }, { "id": "vrb", "phones": "s \u0259 b \u02C8\u0254\u02D0\u0279 d \u026A n \u02CCe\u026A t", "role": "v" }] }, "supplement": { "model": { "bias": 0.4311, "kind": "noun-verb", "weights": { "p1=that": 0.2073, "tp2p1=S+DET": 0.541 } }, "senses": [{ "id": "nou", "phones": "s \u02C8\u028C p l \u0259 m \u0259 n t", "role": "n" }, { "id": "vrb", "phones": "s \u02C8\u028C p l \u0259 m \u02CC\u025B n t", "role": "v" }] }, "suspect": { "model": { "bias": 0.9786, "kind": "noun-verb", "weights": { "tn1=DET": 0.6479 } }, "senses": [{ "id": "adj-nou", "phones": "s \u02C8\u028C s p \u025B k t", "role": "n" }, { "id": "vrb", "phones": "s \u0259 s p \u02C8\u025B k t", "role": "v" }] }, "syndicate": { "model": { "bias": 0, "kind": "noun-verb", "weights": {} }, "senses": [{ "id": "nou", "phones": "s \u02C8\u026A n d \u026A k \u0259 t", "role": "n" }, { "id": "vrb", "phones": "s \u02C8\u026A n d \u026A k \u02CCe\u026A t", "role": "v" }] }, "tear": { "model": { "bias": 3.57, "kind": "noun-verb", "weights": { "cap": 0.1526, "tn1=PREP": 0.8405, "tn1=S": -0.7196, "tn1n2=PUNCT+S": 0.4728, "tn2=DET": 0.7506, "tn2=POS": 0.3234, "tp2=W": 1.1215, "w=a": 1.8769, "w=and": -0.1958, "w=gas": -4.9365, "w=he": 0.0479, "w=on": 1.6189, "w=the": 0.3044, "w=was": 0.2349 } }, "senses": [{ "id": "nou", "phones": "t \u02C8\u026A\u0279", "role": "n" }, { "id": "vrb", "phones": "t \u02C8\u025B\u0279", "role": "v" }] }, "transform": { "model": { "bias": [0, 0], "kind": "senses", "weights": [{}, { "p1=to": 1.1021, "p2=the": -0.1111, "tn1=PUNCT": -0.82, "tn2=W": 0.6763, "tp1=CAP": -0.792, "tp1=MODAL": 0.6594, "tp1=TO": 1.3088, "tp1=W": -0.5971, "tp2=DET": -1.032, "w=and": 0.5308, "w=fourier": -0.6977, "w=into": 2.5148, "w=is": -1.3963, "w=to": 0.5493 }] }, "senses": [{ "id": "nou", "phones": "t \u0279 \u02C8\xE6 n s f \u0254\u02D0\u0279 m", "role": "n" }, { "id": "transform", "phones": "t \u0279 \xE6 n s f \u02C8\u0254\u02D0\u0279 m", "role": "o" }] }, "transplant": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn1=S": -0.4211 } }, "senses": [{ "id": "nou", "phones": "t \u0279 \u02C8\xE6 n s p l \xE6 n t", "role": "n" }, { "id": "vrb", "phones": "t \u0279 \xE6 n s p l \u02C8\xE6 n t", "role": "v" }] }, "transport": { "model": { "bias": -0.0593, "kind": "noun-verb", "weights": { "tn2=CONJ": 0.7197, "tp2=W": -0.2861 } }, "senses": [{ "id": "nou", "phones": "t \u0279 \u02C8\xE6 n s p o\u02D0\u0279 t", "role": "n" }, { "id": "vrb", "phones": "t \u0279 \xE6 n s p \u02C8o\u02D0\u0279 t", "role": "v" }] }, "upset": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "p1=was": 0.1023, "p2=<s>": 0.5889, "tn2=W": 0.1647, "tp1=CAP": 0.076, "tp1n1=BE+PREP": 1.5371, "tp1n1=BE+PUNCT": 1.8533, "tp2=<s>": 0.6883, "tp2=BE": -0.7052, "w=in": 1.865, "w=that": -1.5572, "w=the": 1.5008, "w=was": 0.967, "w=with": -0.0303 } }, "senses": [{ "id": "nou", "phones": "\u02C8\u028C p s \u025B t", "role": "n" }, { "id": "vrb", "phones": "\u0259 p s \u02C8\u025B t", "role": "v" }] }, "use": { "model": { "bias": 0.8581, "kind": "noun-verb", "weights": { "n1=.": -25e-4, "n2=</s>": -0.058, "tn1=W": 1.0201, "tn1n2=PUNCT+</s>": -18e-4, "tn2=</s>": -0.1052, "tp1=S": 1.1462, "tp2=S": 0.6539, "tp2=W": -0.3045, "w=the": 0.9995 } }, "senses": [{ "id": "nou", "phones": "j \u02C8u\u02D0 s", "role": "n" }, { "id": "vrb", "phones": "j \u02C8u\u02D0 z", "role": "v" }] }, "uses": { "model": { "bias": 4.9809, "kind": "noun-verb", "weights": { "tn1=PREP": -2, "tn1=S": 0.3249, "tn1=W": 0.4896, "tn2=PUNCT": 1.0271, "tp1=CAP": 0.1426, "tp2=DET": 0.6556 } }, "senses": [{ "id": "nou", "phones": "j \u02C8u\u02D0 s \u1D7B z", "role": "n" }, { "id": "vrb", "phones": "j \u02C8u\u02D0 z \u1D7B z", "role": "v" }] }, "wind": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "tn1=W": 0.1599, "tn2=S": -1.3768 } }, "senses": [{ "id": "nou", "phones": "w \u02C8\u026A n d", "role": "n" }, { "id": "vrb", "phones": "w \u02C8a\u026A n d", "role": "v" }] }, "winds": { "model": { "bias": 0, "kind": "noun-verb", "weights": { "n1=through": 4.0711, "n1=up": 0.093, "tp2=DET": 0.25, "tp2p1=DET+W": 1.2695, "w=the": 0.5108 } }, "senses": [{ "id": "nou", "phones": "w \u02C8\u026A n d z", "role": "n" }, { "id": "vrb", "phones": "w \u02C8a\u026A n d z", "role": "v" }] }, "wound": { "model": { "bias": [0, -0.8634], "kind": "senses", "weights": [{}, { "n1=on": 0.6389, "n1=up": 2.4926, "tn1=PREP": 0.7285, "tn1=W": 1.3481, "tn2=BE": -0.2465, "tp1=DET": -1.9609, "tp1=S": 0.0614, "tp1=W": -1.7887, "tp1n1=S+PREP": 0.1973, "tp2p1=W+S": 0.0609, "w=a": -0.1854, "w=and": -0.962, "w=coil": 1.3397, "w=of": -0.1924, "w=to": -0.2797, "w=with": 0.9452 }] }, "senses": [{ "id": "nou-vrb", "phones": "w \u02C8u\u02D0 n d", "role": "o" }, { "id": "vrb", "phones": "w \u02C8a\u028A n d", "role": "v" }] } } };

  // ../frontend/dizione/homographs.js
  var CLASSES = [
    ["TO", "to"],
    ["DET", "the a an this that these those every each no some any another such what which whose"],
    ["POSS", "my your his her its our their"],
    ["SUBJ", "i you he she it we they who"],
    ["OBJ", "me him us them"],
    ["MODAL", "will would shall should can could may might must 'll 'd won't can't cannot couldn't shouldn't wouldn't let's please"],
    ["DO", "do does did don't doesn't didn't"],
    ["BE", "am is are was were be been being isn't aren't wasn't weren't i'm you're we're they're he's she's it's that's there's"],
    ["HAVE", "have has had having haven't hasn't hadn't i've you've we've they've"],
    ["NEG", "not never"],
    ["PREP", "of in on at by for from with into onto over under about after before between through during without within against among around across behind beside near upon toward towards as than like"],
    ["CONJ", "and or but nor so yet if because although though while when where unless until since whether"],
    ["ADV", "very too quite rather more most less least also just only even still always often sometimes usually again now then here there soon already almost"]
  ];
  var TAG = new Map(CLASSES.flatMap(([name, list]) => list.split(" ").map((word) => [word, name])));
  var SENTENCE_END = /* @__PURE__ */ new Set([".", "?", "!"]);
  var WINDOW = 6;
  var isAlpha2 = (c) => /\p{L}/u.test(c);
  var isUpper2 = (c) => c !== c.toLowerCase() && c === c.toUpperCase();
  function tag(token) {
    if (token === "<s>" || token === "</s>") return token;
    if (!isAlpha2(token[0])) return /\d/.test(token[0]) ? "NUM" : "PUNCT";
    const lower = token.toLowerCase();
    if (TAG.has(lower)) return TAG.get(lower);
    for (const [ending, name] of [["'s", "POS"], ["ly", "LY"], ["ing", "ING"], ["ed", "ED"]]) {
      if (lower.endsWith(ending) && lower.length > ending.length + 1) return name;
    }
    if (lower.endsWith("s") && !lower.endsWith("ss") && lower.length > 3) return "S";
    return isUpper2(token[0]) ? "CAP" : "W";
  }
  function features(tokens, index) {
    const at = (offset) => {
      const position = index + offset;
      return position >= 0 && position < tokens.length ? tokens[position] : offset < 0 ? "<s>" : "</s>";
    };
    const [p2, p1, n1, n2] = [at(-2), at(-1), at(1), at(2)];
    const [tp2, tp1, tn1, tn2] = [p2, p1, n1, n2].map(tag);
    const out = ["tp1=" + tp1, "tp2=" + tp2, "tn1=" + tn1, "tn2=" + tn2, `tp2p1=${tp2}+${tp1}`, `tp1n1=${tp1}+${tn1}`, `tn1n2=${tn1}+${tn2}`];
    if (index === 0) out.push("first");
    else if (isUpper2(tokens[index][0])) out.push("cap");
    out.push("p1=" + p1.toLowerCase(), "p2=" + p2.toLowerCase(), "n1=" + n1.toLowerCase(), "n2=" + n2.toLowerCase());
    const target = tokens[index].toLowerCase();
    const nearby = new Set(tokens.slice(Math.max(0, index - WINDOW), index + WINDOW + 1).filter((t) => isAlpha2(t[0])).map((t) => t.toLowerCase()));
    nearby.delete(target);
    for (const word of nearby) out.push("w=" + word);
    return out;
  }
  function sentenceOf(tokens, index) {
    let start = index;
    while (start > 0 && !SENTENCE_END.has(tokens[start - 1])) start--;
    let end = index;
    while (end < tokens.length && !SENTENCE_END.has(tokens[end])) end++;
    return [tokens.slice(start, end + 1), index - start];
  }
  var SKIP_FOR_BEFORE = /* @__PURE__ */ new Set(["ADV", "NEG", "LY"]);
  var SKIP_FOR_AUX = /* @__PURE__ */ new Set([...SKIP_FOR_BEFORE, "SUBJ", "OBJ", "DET", "POSS"]);
  function left(tokens, index, skip) {
    for (let position = index - 1; position > Math.max(index - 6, -1); position--) {
      const token = tokens[position];
      if (skip.has(tag(token))) continue;
      const lower = token.toLowerCase();
      return lower.endsWith("'ll") ? "will" : lower.endsWith("'ve") ? "have" : lower;
    }
    return "<s>";
  }
  var RULES = /* @__PURE__ */ new Map();
  for (const [word, sense, conditions] of HOMOGRAPH_RULES) {
    const parsed = conditions.split(";").map((part) => {
      const text = part.trim(), colon = text.indexOf(":");
      const name = colon < 0 ? text : text.slice(0, colon);
      return [name, new Set(colon < 0 ? [] : text.slice(colon + 1).trim().split(/\s+/))];
    });
    if (!RULES.has(word)) RULES.set(word, []);
    RULES.get(word).push([sense, parsed]);
  }
  function holds(condition, words2, tokens, index) {
    if (condition === "first") return index === 0;
    if (condition === "aux") return words2.has(left(tokens, index, SKIP_FOR_AUX));
    if (condition === "before") return words2.has(left(tokens, index, SKIP_FOR_BEFORE));
    if (condition === "next") return words2.has(index + 1 < tokens.length ? tokens[index + 1].toLowerCase() : "</s>");
    return tokens.slice(Math.max(0, index - WINDOW), index + WINDOW + 1).some((token) => words2.has(token.toLowerCase()));
  }
  var PLURAL_BASES = new Set(homographs_data_default.plural_bases);
  function covers(word) {
    return word in homographs_data_default.words || word.endsWith("s") && PLURAL_BASES.has(word.slice(0, -1));
  }
  function scores(word, tokens, index) {
    const entry = homographs_data_default.words[word];
    const names = features(...sentenceOf(tokens, index));
    const model = entry.model;
    if (model.kind === "noun-verb") {
      const sharedNames = names.filter((name) => name[0] === "t" || name === "first" || name === "cap");
      let verb = homographs_data_default.pooled.bias + sharedNames.reduce((sum, name) => sum + (homographs_data_default.pooled.weights[name] ?? 0), 0);
      verb += model.bias + names.reduce((sum, name) => sum + (model.weights[name] ?? 0), 0);
      return entry.senses.map((sense) => sense.role === "v" ? verb : -verb);
    }
    return model.bias.map((bias, i) => bias + names.reduce((sum, name) => sum + (model.weights[i][name] ?? 0), 0));
  }
  function choose(word, tokens, index) {
    const entry = homographs_data_default.words[word];
    const [sentence, position] = sentenceOf(tokens, index);
    for (const [sense, conditions] of RULES.get(word) ?? []) {
      if (conditions.every(([name, words2]) => holds(name, words2, sentence, position))) {
        return entry.senses.find((s) => s.id === sense);
      }
    }
    const list = scores(word, tokens, index);
    let best = 0;
    for (let i = 1; i < list.length; i++) if (list[i] > list[best]) best = i;
    return entry.senses[best];
  }
  function pronounce(word, tokens, index) {
    const ipa = (sense) => sense.phones.replace(/ /g, "");
    if (word in homographs_data_default.words) {
      return { sense: ipa(choose(word, tokens, index)), senses: homographs_data_default.words[word].senses.map(ipa), plural: false };
    }
    const base = word.slice(0, -1), entry = homographs_data_default.words[base];
    const adjective = entry.senses.find((sense) => sense.role === "n");
    const chosen = adjective && adjective.id === "adj" ? entry.senses.find((sense) => sense.role === "v") : choose(base, tokens, index);
    return { sense: ipa(chosen), senses: entry.senses.map(ipa), plural: true };
  }

  // ../frontend/words/kaleTts.js
  function normalizer(lex, Rules = Normalizer) {
    const rules = new Rules((word) => lex.has(word));
    const spell = (letters) => {
      const upper = letters.toUpperCase();
      return upper.length <= 5 && !(upper.length >= 3 && lex.has(letters.toLowerCase())) ? upper : [...upper].join(" ");
    };
    return (text) => dottedNames(spokenText(weekdayAbbrevs(mathMultiply(mathEquations(relativePaths(sizeSuffixes(text))))).replace(/\bDr\.(?=\s*$)/, "Doctor"), rules, spell));
  }
  var DOTTED = /(?<![\p{L}\p{N}.])((?:[\p{L}\p{N}][\p{L}\p{N}_-]*\.)+[\p{Ll}][\p{L}\p{N}_-]*)/gu;
  function dottedNames(text) {
    return text.replace(DOTTED, (name) => {
      const parts = name.split(".");
      if (parts.every((x) => x.length === 1)) return name;
      const end = parts.length - 1;
      if (ENDINGS[parts[end].toLowerCase()] === "letters") parts[end] = parts[end].toUpperCase();
      return parts.join(" dot ");
    });
  }
  function relativePaths(text) {
    return text.replace(/(?<![\p{L}\p{N}_./])(\.\.?)\/(?=[\p{L}\p{N}_~-])/gu, (m, dots) => dots === ".." ? " dot dot slash " : " dot slash ");
  }
  function dropListComma(s) {
    return s.replace(/,(\s+(?:and|or)\s)/gi, (m, tail, at) => {
      const clause = s.slice(0, at).split(/[;:]/).pop();
      const parts = clause.split(",");
      if (parts.length < 2) return m;
      const items = parts.slice(1).map((p) => p.trim().split(/\s+/).filter(Boolean).length);
      return items.every((n) => n >= 1 && n <= 2) ? tail : m;
    });
  }
  function sizeSuffixes(text) {
    const word = { M: "million", B: "billion", K: "thousand", k: "thousand" };
    return text.replace(/(?<![\p{L}\p{N}.])(\d+(?:\.\d+)?)([MBKk])(?![\p{L}\p{N}])/gu, (m, n, s) => `${n} ${word[s]}`).replace(/(?<![\p{L}\p{N}.])(\d+(?:\.\d+)?)[x×](?![\p{L}\p{N}])/gu, (m, n) => `${n} times`).replace(/(?<![\p{L}\p{N}.])(\d+(?: s|\.\d+ ?s))(?![\p{L}\p{N}'’])/gu, (m, n) => `${n.replace(/ ?s$/, "")} sec`);
  }
  var MATH_OPERAND = String.raw`(?:\d+(?:,\d{3})*(?:\.\d+)?|\?)`;
  var MATH_EQUATION = new RegExp(
    String.raw`(?<![\p{L}\p{N}.])${MATH_OPERAND}(?:\s*[+−×÷=-]\s*[-−]?${MATH_OPERAND})+(?![\p{L}\p{N}]|\.\d)`,
    "gu"
  );
  var MATH_SUB = new RegExp(String.raw`(${MATH_OPERAND})\s*[-−]\s*(?=${MATH_OPERAND})`, "g");
  function mathEquations(text) {
    return text.replace(MATH_EQUATION, (expr) => expr.includes("=") ? expr.replace(MATH_SUB, "$1 minus ") : expr);
  }
  var MATH_BEFORE = /(?:what(?:'s|’s| is)|solve|calculate|compute|multiply|find|does|equals|[+\-=÷(])\s*$/i;
  function mathMultiply(text) {
    return text.replace(/(?<![\p{L}\p{N}.])(\d+(?:\.\d+)?)\s?[xX]\s?(\d+(?:\.\d+)?)(?![\p{L}\p{N}])/gu, (m, a, b, at, whole) => {
      const before2 = whole.slice(Math.max(0, at - 24), at);
      const after = whole.slice(at + m.length, at + m.length + 12);
      if (MATH_BEFORE.test(before2) || /^\s*(?:=|\?|equals\b|is\b)/i.test(after)) return `${a} times ${b}`;
      return m;
    });
  }
  var DAY_BEFORE = /\b(?:Mon(?:day)?|Tue(?:s(?:day)?)?|Wed(?:nesday)?|Thu(?:r(?:s(?:day)?)?)?|Fri(?:day)?|Sat(?:urday)?|Sun(?:day)?|on|every|next|last|this)\b(?:,|\s+and|\s+or|\s+to|\s*&|\s*[-–])?\s*$/i;
  var DAY_AFTER = /^(?:\s*,\s*(?:Mon(?:day)?|Tue(?:s(?:day)?)?|Wed(?:nesday)?|Thu(?:r(?:s(?:day)?)?)?|Fri(?:day)?|Sat(?:urday)?|Sun(?:day)?)\b|\s+(?:and|or|to|&)\s+(?:Mon(?:day)?|Tue(?:s(?:day)?)?|Wed(?:nesday)?|Thu(?:r(?:s(?:day)?)?)?|Fri(?:day)?|Sat(?:urday)?|Sun(?:day)?)\b|\s+(?:at\s+\d|\d{1,2}(?::\d{2}|st|nd|rd|th)?\b|Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Sept|Oct|Nov|Dec)\b)/i;
  function weekdayAbbrevs(text) {
    return text.replace(/\b(Sat|Sun)(\.?)(?!['’A-Za-z])/g, (m, day, dot2, at, whole) => {
      const before2 = whole.slice(Math.max(0, at - 28), at);
      const after = whole.slice(at + m.length, at + m.length + 28);
      if (DAY_BEFORE.test(before2) || DAY_AFTER.test(after)) {
        const full = day === "Sat" ? "Saturday" : "Sunday";
        return dot2 && (!after.trim() || /^\s+[A-Z]/.test(after)) ? `${full}.` : full;
      }
      return m;
    });
  }
  var oneWord = (s) => (s.match(/[\p{L}\p{N}]+(?:['’][\p{L}]+)*/gu) ?? []).length === 1;
  function splitNormalized(text, normalize) {
    const norm2 = normalize(text);
    const protectedSpans = [];
    let cursor = 0;
    for (const m of text.matchAll(MATH_EQUATION)) {
      if (!m[0].includes("=") || !m[0].slice(0, -1).includes("?")) continue;
      const eqNorm = normalize(m[0]);
      const at = norm2.indexOf(eqNorm, cursor);
      if (at >= 0) {
        protectedSpans.push([at, at + eqNorm.length]);
        cursor = at + eqNorm.length;
      }
    }
    if (!protectedSpans.length) return norm2.split(/(?<=[.!?…])\s+|\n+/);
    const out = [];
    let start = 0;
    for (const m of norm2.matchAll(/(?<=[.!?…])\s+|\n+/g)) {
      if (protectedSpans.some(([a, b]) => m.index >= a && m.index < b)) continue;
      out.push(norm2.slice(start, m.index));
      start = m.index + m[0].length;
    }
    out.push(norm2.slice(start));
    return out;
  }
  function sentences(text, normalize) {
    const out = [];
    let carry = "";
    for (const part of splitNormalized(text, normalize)) {
      const s = dropListComma(part.trim());
      if (!s) continue;
      const joined = carry ? `${carry} ${s}` : s;
      if (oneWord(joined)) {
        carry = joined;
        continue;
      }
      out.push(joined);
      carry = "";
    }
    if (carry) {
      if (out.length) out[out.length - 1] += ` ${carry}`;
      else out.push(carry);
    }
    return out;
  }
  var HYPHEN_PREFIX = { re: "\u0279\u02CCi\u02D0", de: "d\u02CCi\u02D0", co: "k\u02CCo\u028A", pre: "p\u0279\u02CCi\u02D0", non: "n\u02CC\u0251\u02D0n", sub: "s\u02CC\u028Cb", un: "\u02CC\u028Cn", semi: "s\u02CC\u025Bmi", anti: "\u02CC\xE6nti", multi: "m\u02CC\u028Clti", e: "\u02C8i\u02D0", t: "t\u02C8i\u02D0", x: "\u02C8\u025Bks" };
  var TOKEN = /(?:re|de|co|pre|non|sub|un|semi|anti|multi|e|t|x)-[A-Za-z]+(?:['’][A-Za-z]+)*|lead-(?:free|based)|[A-Za-z]+-minute(?![A-Za-z])|[A-Za-z]+(?:['’][A-Za-z]+)*|[.,!?;:…—–]/gi;
  var PUNCT = { ".": ".", ",": ",", "!": "!", "?": "?", ";": ";", ":": ":", "\u2026": "\u2026", "\u2014": ",", "\u2013": "," };
  var VOWELS = new Set("aeiou\u0251\u0250\u0252\xE6\u0254\u0259\u0258\u025A\u025B\u025C\u025D\u025E\u0268\u026A\u028A\u028C\u0264\xF8\u0153\u0276\u026F\u0275\u0289\u028F\u1D7B");
  var SIBILANT = ["s", "z", "\u0283", "\u0292", "t\u0283", "d\u0292"];
  var VOICELESS = ["p", "t", "k", "f", "\u03B8"];
  var SUFFIXES = [["ly", "li"], ["ness", "n\u0259s"], ["ful", "f\u0259l"], ["less", "l\u0259s"], ["er", "\u025A"], ["est", "\u026Ast"], ["ment", "m\u0259nt"]];
  var LETTER_SOUND = {
    a: "\xE6",
    b: "b",
    c: "k",
    d: "d",
    e: "\u025B",
    f: "f",
    g: "\u0261",
    h: "h",
    i: "\u026A",
    j: "d\u0292",
    k: "k",
    l: "l",
    m: "m",
    n: "n",
    o: "\u0251\u02D0",
    p: "p",
    q: "k",
    r: "\u0279",
    s: "s",
    t: "t",
    u: "\u028C",
    v: "v",
    w: "w",
    x: "ks",
    y: "j",
    z: "z"
  };
  var FOLD = {
    "\xE6": "ae",
    "\xC6": "Ae",
    "\u0153": "oe",
    "\u0152": "Oe",
    "\xF8": "o",
    "\xD8": "O",
    "\xDF": "ss",
    "\u0142": "l",
    "\u0141": "L",
    "\u0111": "d",
    "\u0110": "D",
    "\xF0": "d",
    "\xD0": "D",
    "\xFE": "th",
    "\xDE": "Th",
    "\u0131": "i",
    "\u02BB": "",
    "\u02BC": "",
    "\u2018": "'",
    "`": "'"
  };
  var MAX_LEARNT = 4096;
  function fold(text) {
    let out = "";
    for (const ch of text.normalize("NFD")) {
      if (/\p{Mn}/u.test(ch)) continue;
      out += ch in FOLD ? FOLD[ch] : ch;
    }
    return out.normalize("NFC");
  }
  var norm = (t) => t.replace(/’/g, "'");
  var unstress = (ipa) => ipa.replace(/[ˈˌ]/g, "");
  var secondary = (ipa) => ipa.replace(/ˈ/g, "\u02CC");
  var VOWEL = /eɪ|aɪ|aʊ|ɔɪ|oʊ|əʊ|ɑː|iː|uː|ɜː|ɔː|oː|[aeiouæɑɒɔəɚɛɜɪʊʌᵻɐ]/g;
  var vowelCount = (ipa) => (ipa.match(VOWEL) || []).length;
  var bare = (ipa) => ipa.replace(/[ˈˌː]/g, "");
  function editDistance(x, y) {
    const d = Array.from({ length: y.length + 1 }, (_, j) => j);
    for (let i = 1; i <= x.length; i++) {
      let prev = d[0];
      d[0] = i;
      for (let j = 1; j <= y.length; j++) {
        const t = d[j];
        d[j] = Math.min(d[j] + 1, d[j - 1] + 1, prev + (x[i - 1] !== y[j - 1] ? 1 : 0));
        prev = t;
      }
    }
    return d[y.length];
  }
  function firstSound(ipa) {
    for (const ch of ipa) if (ch !== "\u02C8" && ch !== "\u02CC") return ch;
    return null;
  }
  function stressFirstVowel(ipa) {
    const plain = unstress(ipa);
    const at = [...plain].findIndex((c) => VOWELS.has(c));
    if (at < 0) return plain;
    const cs = [...plain];
    return cs.slice(0, at).join("") + "\u02C8" + cs.slice(at).join("");
  }
  var pluralSuffix = (ipa) => SIBILANT.some((s) => ipa.endsWith(s)) ? "\u1D7Bz" : VOICELESS.some((s) => ipa.endsWith(s)) ? "s" : "z";
  function pastSuffix(ipa) {
    if (ipa.endsWith("t") || ipa.endsWith("d")) return "\u1D7Bd";
    if (VOICELESS.concat(["s", "\u0283", "t\u0283"]).some((s) => ipa.endsWith(s))) return "t";
    return "d";
  }
  function readTable(text, into) {
    const out = into || /* @__PURE__ */ new Map();
    for (const raw of text.split("\n")) {
      const line = raw.replace(/\r$/, "");
      if (!line || line.startsWith("#")) continue;
      const tab = line.indexOf("	");
      if (tab > 0) out.set(line.slice(0, tab), line.slice(tab + 1));
    }
    return out;
  }
  var G2pModel = class {
    constructor(buf) {
      const dv = new DataView(buf);
      let o = 0;
      const i32 = () => {
        const v = dv.getInt32(o, true);
        o += 4;
        return v;
      };
      const magic = String.fromCharCode(dv.getUint8(0), dv.getUint8(1), dv.getUint8(2), dv.getUint8(3));
      o = 4;
      if (magic !== "KG2P") throw new Error("not a g2p model");
      const version = i32();
      if (version !== 1) throw new Error("g2p model version " + version);
      const e = i32(), h = i32(), d = i32(), nIn = i32(), nOut = i32(), maxOut = i32();
      const inputs = Array.from({ length: nIn }, i32);
      const outputs = Array.from({ length: nOut }, i32);
      const take = (n) => {
        const a = new Float64Array(n);
        for (let k = 0; k < n; k++) {
          a[k] = half(dv.getUint16(o, true));
          o += 2;
        }
        return a;
      };
      const gru = (cols, size) => ({ wIh: take(3 * size * cols), wHh: take(3 * size * size), bIh: take(3 * size), bHh: take(3 * size) });
      this.w = {
        embIn: take(nIn * e),
        encFwd: gru(e, h),
        encBwd: gru(e, h),
        initW: take(d * 2 * h),
        initB: take(d),
        embOut: take(nOut * e),
        dec: gru(e + 2 * h, d),
        attW: take(2 * h * d),
        combW: take(d * (d + 2 * h)),
        combB: take(d),
        outW: take(nOut * d),
        outB: take(nOut)
      };
      if (o !== buf.byteLength) throw new Error("g2p model size");
      this.e = e;
      this.h = h;
      this.d = d;
      this.maxOut = maxOut;
      this.outputs = outputs;
      this.inIndex = /* @__PURE__ */ new Map();
      for (let i = 1; i < nIn; i++) this.inIndex.set(inputs[i], i);
    }
    phonemes(word) {
      if (!word || word.length > 24) return null;
      const { e, h, d, w } = this;
      const ids = [];
      for (const ch of word) {
        const id = this.inIndex.get(ch.codePointAt(0));
        if (id === void 0) return null;
        ids.push(id);
      }
      const n = ids.length;
      const emb = (table, id) => table.subarray(id * e, id * e + e);
      const fwd = [], bwd = [];
      let hf = new Float64Array(h);
      for (let t = 0; t < n; t++) {
        hf = gruCell(w.encFwd, emb(w.embIn, ids[t]), hf, h);
        fwd[t] = hf;
      }
      let hb = new Float64Array(h);
      for (let t = n - 1; t >= 0; t--) {
        hb = gruCell(w.encBwd, emb(w.embIn, ids[t]), hb, h);
        bwd[t] = hb;
      }
      const enc = fwd.map((f, t) => cat(f, bwd[t]));
      let state = matVec(w.initW, cat(hf, hb), d, w.initB).map(Math.tanh);
      let ctx = new Float64Array(2 * h);
      let prev = 1;
      let out = "";
      const limit = Math.min(this.maxOut, 3 * n + 6);
      let ended = false;
      for (let step = 0; step < limit; step++) {
        state = gruCell(w.dec, cat(emb(w.embOut, prev), ctx), state, d);
        const q = matVec(w.attW, state, 2 * h, null);
        const scores2 = enc.map((v) => dot(v, q));
        const top = Math.max(...scores2);
        let total = 0;
        for (let t = 0; t < n; t++) {
          scores2[t] = Math.exp(scores2[t] - top);
          total += scores2[t];
        }
        ctx = new Float64Array(2 * h);
        for (let k = 0; k < 2 * h; k++) {
          let s = 0;
          for (let t = 0; t < n; t++) s += scores2[t] * enc[t][k];
          ctx[k] = s / total;
        }
        const hid = matVec(w.combW, cat(state, ctx), d, w.combB).map(Math.tanh);
        const logits = matVec(w.outW, hid, this.outputs.length, w.outB);
        let best = 0;
        for (let i = 1; i < logits.length; i++) if (logits[i] > logits[best]) best = i;
        if (best === 2) {
          ended = true;
          break;
        }
        if (best < 3) return null;
        out += String.fromCodePoint(this.outputs[best]);
        prev = best;
      }
      if (!ended) return null;
      return [...out].some((c) => VOWELS.has(c)) ? out : null;
    }
  };
  function half(v) {
    const sign = v & 32768 ? -1 : 1, ex = v >> 10 & 31, m = v & 1023;
    if (ex === 0) return Math.fround(sign * m * Math.pow(2, -24));
    if (ex === 31) return m === 0 ? sign * Infinity : NaN;
    return Math.fround(sign * (1 + m / 1024) * Math.pow(2, ex - 15));
  }
  function cat(a, b) {
    const o = new Float64Array(a.length + b.length);
    o.set(a);
    o.set(b, a.length);
    return o;
  }
  function dot(a, b) {
    let s = 0;
    for (let i = 0; i < a.length; i++) s += a[i] * b[i];
    return s;
  }
  function row(m, k, x, bias) {
    let s = bias;
    const base = k * x.length;
    for (let i = 0; i < x.length; i++) s += m[base + i] * x[i];
    return s;
  }
  function matVec(m, x, rows, b) {
    const o = new Float64Array(rows);
    for (let k = 0; k < rows; k++) o[k] = row(m, k, x, b ? b[k] : 0);
    return o;
  }
  var sigmoid = (x) => 1 / (1 + Math.exp(-x));
  function gruCell(g, x, hPrev, size) {
    const out = new Float64Array(size);
    for (let k = 0; k < size; k++) {
      const r = sigmoid(row(g.wIh, k, x, g.bIh[k]) + row(g.wHh, k, hPrev, g.bHh[k]));
      const z = sigmoid(row(g.wIh, size + k, x, g.bIh[size + k]) + row(g.wHh, size + k, hPrev, g.bHh[size + k]));
      const c = Math.tanh(row(g.wIh, 2 * size + k, x, g.bIh[2 * size + k]) + r * row(g.wHh, 2 * size + k, hPrev, g.bHh[2 * size + k]));
      out[k] = (1 - z) * c + z * hPrev[k];
    }
    return out;
  }
  var LEAD_METAL_NEXT = /* @__PURE__ */ new Set(["pipe", "pipes", "piping", "paint", "paints", "poisoning", "poison", "acid", "oxide", "ore", "solder"]);
  var LEAD_METAL_PREV = /* @__PURE__ */ new Set(["red", "white"]);
  var LEAD_MADE = /* @__PURE__ */ new Set(["made", "built", "cast", "lined", "pipes", "weights"]);
  var READ_HABIT = /* @__PURE__ */ new Set(["daily", "often", "nightly", "regularly"]);
  var PERIODS = /* @__PURE__ */ new Set(["day", "night", "morning", "evening", "week", "weekend"]);
  var PAST = /* @__PURE__ */ new Set(["yesterday", "ago", "last", "had", "has", "have", "was", "were", "already"]);
  var MINUTE_ADJ_NEXT = /* @__PURE__ */ new Set(["detail", "details", "amount", "amounts", "trace", "traces", "fraction", "fractions", "particle", "particles", "difference", "differences", "change", "changes", "quantity", "quantities", "scale", "crack", "cracks", "speck", "specks", "organism", "organisms", "creature", "creatures", "piece", "pieces", "part", "parts", "drop", "drops", "grain", "grains", "insect", "insects"]);
  var MINUTE_TIME_PREV = /* @__PURE__ */ new Set(["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety", "hundred", "thousand", "million", "per", "every", "each", "last", "next", "first", "second", "third", "final", "half", "quarter", "single", "another", "this", "that"]);
  function senseRule(low, words2, i) {
    const at = (k) => k >= 0 && k < words2.length ? words2[k] : "";
    if (low === "lead") {
      const metal = LEAD_METAL_NEXT.has(at(i + 1)) || LEAD_METAL_PREV.has(at(i - 1)) || (at(i - 1) === "of" || at(i - 1) === "from") && LEAD_MADE.has(at(i - 2));
      return metal ? "l\u02C8\u025Bd" : "l\u02C8i\u02D0d";
    }
    if (low === "read") {
      if (at(i - 1) === "well") return "\u0279\u02C8\u025Bd";
      const habit = READ_HABIT.has(at(i + 1)) || (at(i + 1) === "every" || at(i + 1) === "each") && PERIODS.has(at(i + 2));
      if (habit && !words2.some((x) => PAST.has(x))) return "\u0279\u02C8i\u02D0d";
    }
    if (low === "minute" && MINUTE_ADJ_NEXT.has(at(i + 1)) && !MINUTE_TIME_PREV.has(at(i - 1))) {
      return "ma\u026An\u02C8u\u02D0t";
    }
    return null;
  }
  function letterRun(tokens, start) {
    let letters = "", i = start;
    while (i < tokens.length && /^[A-Z]$/.test(tokens[i])) letters += tokens[i++];
    const plural = i < tokens.length && /^[A-Z]['’]s$/.test(tokens[i]);
    if (plural) letters += tokens[i++][0];
    return letters.length >= 2 ? { letters, plural, end: i } : null;
  }
  var FINAL_STRONG = { the: "\xF0\u02C8\u028C", a: "\u02C8e\u026A", an: "\u02C8\xE6n", to: "t\u02C8u\u02D0" };
  var G2p = class {
    constructor(lex, model) {
      this.lex = lex;
      this.model = model || null;
      this.learnt = /* @__PURE__ */ new Map();
    }
    phonemes(sentence) {
      const tokens = fold(sentence).match(TOKEN) || [];
      const out = [];
      let skip = false;
      for (let i = 0; i < tokens.length; i++) {
        const tok = tokens[i];
        const mark = PUNCT[tok];
        if (mark !== void 0) {
          if (out.length) out[out.length - 1] += mark;
          continue;
        }
        if (skip) {
          skip = false;
          continue;
        }
        const run = letterRun(tokens, i);
        if (run !== null) {
          const ipa2 = this.spell(run.letters);
          out.push(run.plural ? ipa2 + pluralSuffix(ipa2) : ipa2);
          i = run.end - 1;
          continue;
        }
        const nx = tokens[i + 1];
        const next = nx !== void 0 && !(nx in PUNCT) ? nx : null;
        if (next !== null) {
          const pair = this.lex.get(norm(tok).toLowerCase() + " " + norm(next).toLowerCase());
          if (pair !== void 0) {
            out.push(pair);
            skip = true;
            continue;
          }
        }
        const reading = this.homograph(tok, tokens, i);
        if (reading !== null) {
          out.push(reading);
          continue;
        }
        const dash = tok.indexOf("-");
        if (dash > 0) {
          const pfx = tok.slice(0, dash).toLowerCase(), stem = tok.slice(dash + 1);
          if (pfx === "lead" && (stem.toLowerCase() === "free" || stem.toLowerCase() === "based")) {
            out.push("l\u02C8\u025Bd");
            const sIpa2 = this.word(stem, next);
            if (sIpa2) out.push(sIpa2);
            continue;
          }
          if (stem.toLowerCase() === "minute" && !(pfx in HYPHEN_PREFIX)) {
            const lIpa = this.word(tok.slice(0, dash), "minute");
            if (lIpa) out.push(lIpa);
            out.push("m\u02C8\u026An\u026At");
            continue;
          }
          const pIpa = this.hyphenPrefix(pfx), sIpa = this.word(stem, next);
          if (pIpa && sIpa) {
            out.push(pIpa + (pIpa.includes("\u02C8") ? secondary(sIpa) : sIpa));
            continue;
          }
        }
        const ipa = this.word(tok, next);
        if (ipa) out.push(ipa);
      }
      return out.join(" ");
    }
    hyphenPrefix(pfx) {
      return HYPHEN_PREFIX[pfx] ?? null;
    }
    /**
     * A homograph as its sentence means it ("read" past or present). The word list's own pronunciation stays
     * when it is already the reading meant (closest to it of the readings); dizione's when another one is.
     */
    homograph(token, tokens, index) {
      const w = norm(token), low = w.toLowerCase();
      if (w.length > 1 && w === w.toUpperCase()) return null;
      const ruled = senseRule(low, tokens.map((t) => norm(t).toLowerCase()), index);
      if (ruled !== null) return ruled;
      if (!covers(low)) return null;
      const { sense, senses, plural } = pronounce(low, tokens.map(norm), index);
      const own = this.lex.get(plural ? low.slice(0, -1) : low);
      let ipa = sense;
      if (own !== void 0) {
        const nearest = senses.reduce((best, s) => distance(s, own) < distance(best, own) ? s : best);
        if (nearest === sense) ipa = own;
      }
      return plural ? ipa + pluralSuffix(ipa) : ipa;
    }
    word(token, next) {
      const lex = this.lex;
      const w = norm(token), low = w.toLowerCase();
      if (w.length > 1 && w === w.toUpperCase()) {
        if (lex.has(w)) return lex.get(w);
        if (!(w.length >= 3 && lex.has(low)) && w.length <= 5) return this.spell(w);
      }
      if (next !== null) {
        const following = this.lookup(norm(next).toLowerCase(), null);
        const variant = lex.get(low + "#v");
        if (following != null && variant !== void 0) {
          const f = firstSound(following);
          if (f !== null && VOWELS.has(f)) return variant;
        }
      } else if (low in FINAL_STRONG) {
        return FINAL_STRONG[low];
      }
      return this.lookup(low, w);
    }
    lookup(low, original) {
      const lex = this.lex;
      if (lex.has(low)) return this.letters(low, lex.get(low));
      if (original === null) return null;
      for (const [cands, suffix] of this.stems(low)) {
        for (const stem of cands) {
          const base = lex.get(stem);
          if (base) return base + suffix(base);
        }
      }
      if (low.length === 1) return lex.has(low) ? lex.get(low) : LETTER_SOUND[low] ?? "";
      const said = this.fallback(low);
      if (said != null) return this.joined(low, said) ?? said;
      for (let i = 3; i < low.length - 2; i++) {
        const a = lex.get(low.slice(0, i)), b = lex.get(low.slice(i));
        if (a !== void 0 && b !== void 0) return a + secondary(b);
      }
      return this.chunks(low);
    }
    /**
     * A guessed word of 6+ letters that is two listed words (3+ letters each, the first such split) read as the guess plus one
     * stray vowel ("barcode" guessed as "barcodey"): the two words instead. Any other difference keeps the guess, which reads
     * prefixes, -able words and names better than a split (17 Nov, 40,000-word sweep: 9 changes, none worse).
     */
    joined(low, said) {
      if (low.length < 6) return null;
      for (let i = 3; i <= low.length - 3; i++) {
        const a = this.lex.get(low.slice(0, i)), b = this.lex.get(low.slice(i));
        if (a === void 0 || b === void 0) continue;
        const two = a + secondary(b);
        return vowelCount(said) > vowelCount(two) && editDistance(bare(said), bare(two)) <= 1 ? two : null;
      }
      return null;
    }
    fallback(low) {
      if (!this.model) return null;
      if (this.learnt.has(low)) return this.learnt.get(low);
      const said = this.model.phonemes(low);
      if (this.learnt.size >= MAX_LEARNT) this.learnt.clear();
      this.learnt.set(low, said);
      return said;
    }
    stems(w) {
      const out = [];
      const doubled = (b) => b.length > 2 && b[b.length - 1] === b[b.length - 2] ? [b.slice(0, -1)] : [];
      if (w.endsWith("'s")) out.push([[w.slice(0, -2)], pluralSuffix]);
      if (w.endsWith("s'")) out.push([[w.slice(0, -1)], () => ""]);
      if (w.endsWith("ies") && w.length > 4) out.push([[w.slice(0, -3) + "y"], pluralSuffix]);
      if (w.endsWith("es") && w.length > 3) out.push([[w.slice(0, -2)], pluralSuffix]);
      if (w.endsWith("s") && !w.endsWith("ss") && w.length > 3) out.push([[w.slice(0, -1)], pluralSuffix]);
      if (w.endsWith("ied") && w.length > 4) out.push([[w.slice(0, -3) + "y"], pastSuffix]);
      if (w.endsWith("ed") && w.length > 4) {
        const b = w.slice(0, -2);
        out.push([[b, b + "e", ...doubled(b)], pastSuffix]);
      }
      if (w.endsWith("ing") && w.length > 5) {
        const b = w.slice(0, -3);
        out.push([[b, b + "e", ...doubled(b)], () => "\u026A\u014B"]);
      }
      for (const [suf, sound] of SUFFIXES) {
        if (w.endsWith(suf) && w.length > suf.length + 2) {
          const b = w.slice(0, -suf.length);
          out.push([[b, ...b.endsWith("i") ? [b.slice(0, -1) + "y"] : [], ...doubled(b)], () => sound]);
        }
      }
      return out;
    }
    /** A listed reading that is only the word's letter names (CMUdict's "cpu", "pr", "url") is spelled as "GPU" is. */
    letters(low, ipa) {
      if (low.length < 2 || low.length > 5 || low === "ok" || !/^[a-z]+$/.test(low)) return ipa;
      const names = [...low].map((c) => c === "a" ? "e\u026A" : bare(this.lex.get(c) ?? "")).join("");
      return bare(ipa) === names ? this.spell(low) : ipa;
    }
    spell(word) {
      const parts = [...word].map((ch) => {
        const c = ch.toLowerCase();
        return c === "a" ? "\u02C8e\u026A" : c === "i" ? "\u02C8a\u026A" : this.lex.get(c) ?? LETTER_SOUND[c] ?? "";
      }).filter((p) => p);
      if (!parts.length) return "";
      return [...parts.slice(0, -1).map(secondary), parts[parts.length - 1]].join(" ");
    }
    chunks(w) {
      let out = "", i = 0;
      while (i < w.length) {
        let matched = false;
        for (let j = w.length; j > i + 2; j--) {
          const piece = this.lex.get(w.slice(i, j));
          if (piece) {
            out += unstress(piece);
            i = j;
            matched = true;
            break;
          }
        }
        if (!matched) {
          out += LETTER_SOUND[w[i]] ?? "";
          i++;
        }
      }
      return stressFirstVowel(out);
    }
  };
  function distance(a, b) {
    const x = [...a], y = [...b];
    let row2 = Array.from({ length: y.length + 1 }, (_, j) => j);
    for (let i = 1; i <= x.length; i++) {
      const next = [i];
      for (let j = 1; j <= y.length; j++) next[j] = Math.min(row2[j] + 1, next[j - 1] + 1, row2[j - 1] + (x[i - 1] === y[j - 1] ? 0 : 1));
      row2 = next;
    }
    return row2[y.length];
  }

  // ../frontend/words/enGb.js
  var UNITS2 = [
    "a\u026A\u0259",
    "a\u028A\u0259",
    "e\u026A",
    "a\u026A",
    "a\u028A",
    "\u0254\u026A",
    "o\u028A",
    "\u0259\u028A",
    "\u026A\u0259",
    "\u025B\u0259",
    "\u028A\u0259",
    "\u0251\u02D0",
    "\u0254\u02D0",
    "\u025C\u02D0",
    "i\u02D0",
    "u\u02D0",
    "o\u02D0",
    "t\u0283",
    "d\u0292",
    "n\u0329",
    "l\u0329",
    "m\u0329"
  ];
  var VOWEL_CH = new Set("aeiou\u0251\u0250\u0252\xE6\u0254\u0259\u025B\u025C\u026A\u028A\u028C\u1D7B\u025A\u025D");
  var isVowel = (t) => t === "n\u0329" || t === "l\u0329" || t === "m\u0329" || VOWEL_CH.has([...t][0]);
  function units(ipa) {
    const out = [];
    let stress = "";
    for (let i = 0; i < ipa.length; ) {
      const ch = ipa[i];
      if (ch === "\u02C8" || ch === "\u02CC") {
        stress = ch;
        i++;
        continue;
      }
      let t = UNITS2.find((u) => ipa.startsWith(u, i)) ?? ch;
      i += t.length;
      const v = isVowel(t);
      out.push({ t, v, s: v ? stress : "" });
      if (!v && stress) {
        out[out.length - 1].s = stress;
      }
      stress = "";
    }
    return out;
  }
  var join = (us) => us.map((u) => u.s + u.t).join("");
  function spellings(word, n) {
    const groups = [...word.matchAll(/[aeiou]+|(?<=[^aeiou])y(?![aeiou])|^y(?![aeiou])/g)].map((m) => ({ at: m.index, g: m[0] }));
    const cands = [groups];
    if (groups.length && /[^aeiou](e|es|ed|e's)$/.test(word) && groups[groups.length - 1].g === "e") cands.push(groups.slice(0, -1));
    for (const g of cands) if (g.length === n) return g;
    return null;
  }
  var BATH = new Set("after afternoon afterwards answer ask aunt banana basket bath bathroom blast branch brass can't cask cast castle caste chance chancellor chant clasp class classroom command commander craft crafty dance dancer demand disaster draft draught example fast fasten flask gasp ghastly giraffe glance glass graft grant grasp grass half calf laugh laughter last mask master nasty pass passport past pastor pastoral path plant plaster prance raft rafter rasp rather shaft slant staff task vast advance advantage disadvantage enhance photograph telegraph paragraph autograph graph sample command commando contrast forecast broadcast outcast downcast overcast aghast mast lance france bypass surpass trespass".split(" "));
  var BATH_END = [
    "",
    "s",
    "es",
    "ed",
    "ing",
    "er",
    "ers",
    "y",
    "ier",
    "iest",
    "ly",
    "ful",
    "room",
    "rooms",
    "hopper",
    "hoppers",
    "land",
    "lands",
    "work",
    "time",
    "word",
    "words",
    "ship"
  ];
  function isBath(word) {
    if (BATH.has(word)) return true;
    for (const e of BATH_END) if (e && word.endsWith(e) && BATH.has(word.slice(0, -e.length))) return true;
    for (const e of ["ed", "ing", "er", "es"]) if (word.endsWith(e) && BATH.has(word.slice(0, -e.length) + "e")) return true;
    return false;
  }
  var STRUT_R = /(?:^|[^aeiou])(?:urr|orr|our)(?=[aeiouy])/;
  var NURSE_R = /^(?:fur|pur|blur|slur|spur|whir|stir|sir|occur|recur|concur|incur|demur|prefer|confer|refer|infer|defer|deter|transfer|aver|inter|astir|chirr|shirr|err)/;
  function toBritish(ipa, key = "") {
    if (!ipa) return ipa;
    const linking = key.endsWith("#v");
    const word = key.replace(/#v$/, "").toLowerCase().replace(/’/g, "'");
    const letters = word.replace(/[^a-z' ]/g, "");
    let u = units(ipa);
    const vi = u.map((x, i) => x.v ? i : -1).filter((i) => i >= 0);
    const sp = spellings(letters.replace(/'/g, ""), vi.length);
    const plain = letters.replace(/'/g, "");
    if (sp) vi.forEach((i, k) => {
      u[i].g = sp[k].g + (plain[sp[k].at + sp[k].g.length] === "w" ? "w" : "");
      u[i].at = sp[k].at;
      u[i].len = sp[k].g.length;
    });
    const hasT = /t/.test(letters), hasD = /d/.test(letters);
    const vowelBefore = (i) => {
      for (let j = i - 1; j >= 0; j--) if (u[j].v) return u[j];
      return null;
    };
    const vowelAfter = (i) => {
      for (let j = i + 1; j < u.length; j++) if (u[j].v) return u[j];
      return null;
    };
    for (let i = 0; i < u.length; i++) {
      const x = u[i], prev = u[i - 1], next = u[i + 1], after = u[i + 2], g = x.g;
      if (x.t === "\u027E") {
        let d = hasD && !hasT;
        const a = vowelBefore(i), b = vowelAfter(i);
        if (a && b && a.at !== void 0 && b.at !== void 0) {
          const mid = plain.slice(a.at + a.len, b.at);
          d = !/t/.test(mid) && /d/.test(mid);
        }
        x.t = d ? "d" : "t";
      } else if (x.t === "\u1D7B") x.t = "\u026A";
      else if (x.t === "o\u028A") x.t = "\u0259\u028A";
      else if (x.t === "\u025D") x.t = "\u025C\u02D0";
      else if (x.t === "\u0254") x.t = "\u0252";
      else if (x.t === "\u0251\u02D0" || x.t === "\u0251") {
        const rCons = next && next.t === "\u0279" && !(after && after.v);
        if (!rCons) {
          const lot = g ? /o/.test(g) || g === "a" && prev && prev.t === "w" : /o/.test(letters) && !/a/.test(letters);
          if (lot) x.t = "\u0252";
          else x.t = "\u0251\u02D0";
        }
      } else if (x.t === "\xE6" && x.s && isBath(word) && next && "fs\u03B8nml\u014B".includes(next.t)) x.t = "\u0251\u02D0";
      else if ((x.t === "u\u02D0" || (x.t === "\u028A" || x.t === "\u028A\u0259") && next && next.t === "\u0279") && x.s && prev && ["t", "d", "n", "\u03B8"].includes(prev.t) && g && ["u", "ue", "ew", "eu", "ui", "ewe"].includes(g) && !(u[i - 2] && u[i - 2].t === "j")) {
        u.splice(i, 0, { t: "j", v: false, s: "" });
        i++;
      }
    }
    if (/(ary|ory|ery|aries|ories|eries)$/.test(word)) {
      for (let i = u.length - 3; i >= 0; i--) {
        if (u[i].v && u[i].s !== "\u02C8" && ["\u025B", "\u0254\u02D0", "o\u02D0", "e\u026A"].includes(u[i].t) && u[i + 1].t === "\u0279" && u[i + 2] && ["i", "i\u02D0", "\u026A"].includes(u[i + 2].t)) {
          u[i] = { t: "\u0259", v: true, s: "" };
          break;
        }
      }
    }
    if (/[^aeiou]ile[sd]?$/.test(word)) {
      let k = u.length - 1;
      if (k > 0 && ["z", "d"].includes(u[k].t)) k--;
      if (u[k].t === "l\u0329" && !u[k].s) u.splice(k, 1, { t: "a\u026A", v: true, s: "" }, { t: "l", v: false, s: "" });
      else if (u[k].t === "l" && u[k - 1] && ["\u0259", "\u026A", "\u028C"].includes(u[k - 1].t) && !u[k - 1].s) u[k - 1].t = "a\u026A";
    }
    if (/i[sz]ation/.test(word)) {
      for (let i = 1; i + 2 < u.length; i++) if (["\u0259", "\u026A"].includes(u[i].t) && !u[i].s && u[i + 1].t === "z" && u[i + 2].t === "e\u026A") u[i].t = "a\u026A";
    }
    const out = [];
    for (let i = 0; i < u.length; i++) {
      const x = u[i], next = u[i + 1];
      if (x.t === "\u025A") {
        const keep = next && next.t === "\u0279" && (u[i + 2] && u[i + 2].v || linking && i + 2 === u.length);
        out.push({ ...x, t: "\u0259" });
        if (keep) {
          out.push(next);
          i++;
        } else if (next && next.t === "\u0279" && !(u[i + 2] && u[i + 2].v)) i++;
        continue;
      }
      if (x.t === "\u0279" && out.length && out[out.length - 1].v) {
        const nx = u[i + 1];
        const p = out[out.length - 1];
        const beforeVowel = nx && nx.v || linking && !nx;
        if (beforeVowel) {
          if (p.t === "o\u02D0") p.t = "\u0254\u02D0";
          const rr = p.at !== void 0 && plain.slice(p.at + p.len, p.at + p.len + 2) === "rr";
          if (nx && p.t === "\u025B" && p.g === "a" && rr) p.t = "\xE6";
          else if (nx && p.t === "\u025B" && ["ai", "ae", "ea", "ei"].includes(p.g)) p.t = "\u025B\u0259";
          else if (nx && p.t === "\u026A" && ["e", "ea", "ee", "ie", "eo", "ei"].includes(p.g) && !rr) p.t = "\u026A\u0259";
          else if (nx && p.t === "\u028A" && ["u", "ou", "oo"].includes(p.g) && !rr) p.t = "\u028A\u0259";
          if (p.t === "\u025C\u02D0" && STRUT_R.test(word) && !NURSE_R.test(word)) p.t = "\u028C";
          if (!nx) {
            p.t = { "\u0251\u02D0": "\u0251\u02D0", "\u0254\u02D0": "\u0254\u02D0", "\u025B": "\u025B\u0259", "\u026A": "\u026A\u0259", "\u028A": "\u028A\u0259", "i\u02D0": "\u026A\u0259", "u\u02D0": "\u028A\u0259", "e\u026A": "\u025B\u0259", "a\u026A": "a\u026A\u0259", "a\u028A": "a\u028A\u0259" }[p.t] ?? p.t;
          }
          out.push(x);
          continue;
        }
        p.t = {
          "\u0251\u02D0": "\u0251\u02D0",
          "\u0251": "\u0251\u02D0",
          "\u0254\u02D0": "\u0254\u02D0",
          "o\u02D0": "\u0254\u02D0",
          "\u0252": "\u0254\u02D0",
          "\u025B": "\u025B\u0259",
          "e\u026A": "\u025B\u0259",
          "\u026A": "\u026A\u0259",
          "i\u02D0": "\u026A\u0259",
          "\u028A": "\u028A\u0259",
          "u\u02D0": "\u028A\u0259",
          "a\u026A": "a\u026A\u0259",
          "a\u028A": "a\u028A\u0259",
          "\u025C\u02D0": "\u025C\u02D0",
          "\u0259": "\u0259",
          "\u028C": "\u025C\u02D0",
          "\u0250": "\u0259",
          "\xE6": "\u0251\u02D0"
        }[p.t] ?? p.t;
        continue;
      }
      out.push(x);
    }
    return join(out);
  }
  var BritishG2p = class extends G2p {
    homograph(token, tokens, index) {
      const r = super.homograph(token, tokens, index);
      return r === null ? r : toBritish(r, token.toLowerCase());
    }
    hyphenPrefix(pfx) {
      const r = super.hyphenPrefix(pfx);
      return r === null ? r : toBritish(r, pfx);
    }
    chunks(w) {
      return toBritish(super.chunks(w), w);
    }
    fallback(low) {
      const r = super.fallback(low);
      return r == null ? r : toBritish(r, low);
    }
    // en-gb g2p.bin is British already (no-op)
  };
  var BritishNormalizer = class extends Normalizer {
    date(month, day, yearText) {
      return super.date(month, day, yearText, true);
    }
    slashDate(match) {
      this.dayFirst = true;
      return super.slashDate(match);
    }
    shortDate(match) {
      this.dayFirst = true;
      return super.shortDate(match);
    }
  };
  function britishNormalizer(lex) {
    return normalizer(lex, BritishNormalizer);
  }

  // ../frontend/ettin.js
  var GPT2_SPLIT = /'s|'t|'re|'ve|'m|'ll|'d| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+(?!\S)|\s+/gu;
  function bytesToUnicode() {
    const bs = [];
    for (let i = 33; i <= 126; i++) bs.push(i);
    for (let i = 161; i <= 172; i++) bs.push(i);
    for (let i = 174; i <= 255; i++) bs.push(i);
    const cs = bs.slice();
    let n = 0;
    for (let b = 0; b < 256; b++) if (!bs.includes(b)) {
      bs.push(b);
      cs.push(256 + n);
      n++;
    }
    const m = new Array(256);
    bs.forEach((b, i) => {
      m[b] = String.fromCharCode(cs[i]);
    });
    return m;
  }
  var B2U = bytesToUnicode();
  function utf8(str) {
    const out = [];
    for (const ch of str) {
      const c = ch.codePointAt(0);
      if (c < 128) out.push(c);
      else if (c < 2048) out.push(192 | c >> 6, 128 | c & 63);
      else if (c < 65536) out.push(224 | c >> 12, 128 | c >> 6 & 63, 128 | c & 63);
      else out.push(240 | c >> 18, 128 | c >> 12 & 63, 128 | c >> 6 & 63, 128 | c & 63);
    }
    return out;
  }
  var EttinTokenizer = class {
    /** json: the parsed tokenizer.json */
    constructor(json) {
      const m = json.model;
      this.vocab = m.vocab;
      this.ranks = new Map((m.merges ?? []).map((x, i) => [Array.isArray(x) ? x.join(" ") : x, i]));
      const added = Object.fromEntries((json.added_tokens ?? []).map((t) => [t.content, t.id]));
      this.cls = added["[CLS]"] ?? this.vocab["[CLS]"];
      this.sep = added["[SEP]"] ?? this.vocab["[SEP]"];
      const plain = (json.added_tokens ?? []).filter((t) => !t.special).map((t) => t.content).sort((a, b) => b.length - a.length);
      this.added = new Map((json.added_tokens ?? []).filter((t) => !t.special).map((t) => [t.content, t.id]));
      this.addedRe = plain.length ? new RegExp(plain.map((x) => x.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|"), "gu") : null;
      this.cache = /* @__PURE__ */ new Map();
    }
    _bpe(word) {
      const hit = this.cache.get(word);
      if (hit) return hit;
      let parts = [...word];
      while (parts.length > 1) {
        let best = -1, bestRank = Infinity;
        for (let i = 0; i < parts.length - 1; i++) {
          const r = this.ranks.get(parts[i] + " " + parts[i + 1]);
          if (r !== void 0 && r < bestRank) {
            bestRank = r;
            best = i;
          }
        }
        if (best < 0) break;
        parts = [...parts.slice(0, best), parts[best] + parts[best + 1], ...parts.slice(best + 2)];
      }
      if (this.cache.size < 5e4) this.cache.set(word, parts);
      return parts;
    }
    /** text -> { ids, offsets: [[start, end), ...] in code points of the NFC text, [0,0] for [CLS] / [SEP] } */
    encode(text) {
      const t = text.normalize("NFC"), cps = Array.from(t);
      const cpAtUnit = [];
      {
        let u = 0;
        cps.forEach((c, i) => {
          for (let k = 0; k < c.length; k++) cpAtUnit[u + k] = i;
          u += c.length;
        });
        cpAtUnit[t.length] = cps.length;
      }
      const ids = [this.cls], offsets = [[0, 0]];
      const plain = (seg, segStart) => {
        for (const mt of seg.matchAll(GPT2_SPLIT)) {
          const piece = mt[0], startCp = cpAtUnit[segStart + mt.index];
          const pcs = Array.from(piece), byteCp = [];
          pcs.forEach((c, i) => {
            for (const _ of utf8(c)) byteCp.push(i);
          });
          const word = utf8(piece).map((b) => B2U[b]).join("");
          let at = 0;
          for (const sub2 of this._bpe(word)) {
            const n = [...sub2].length, id = this.vocab[sub2], a = byteCp[at], b = byteCp[at + n - 1] + 1;
            at += n;
            if (id !== void 0) {
              ids.push(id);
              offsets.push([startCp + a, startCp + b]);
            }
          }
        }
      };
      let last = 0;
      if (this.addedRe) for (const mt of t.matchAll(this.addedRe)) {
        if (mt.index > last) plain(t.slice(last, mt.index), last);
        ids.push(this.added.get(mt[0]));
        offsets.push([cpAtUnit[mt.index], cpAtUnit[mt.index + mt[0].length]]);
        last = mt.index + mt[0].length;
      }
      if (last < t.length) plain(t.slice(last), last);
      ids.push(this.sep);
      offsets.push([0, 0]);
      return { ids, offsets };
    }
  };
  function ettinInputs(tok, text, prev = "") {
    const words2 = text.split(/\s+/).filter(Boolean), pre = prev && prev.trim() ? prev.trim() + " " : "";
    const full = pre + words2.join(" "), { ids, offsets } = tok.encode(full);
    const S = ids.length, W = Math.max(words2.length, 1), wavg = new Float32Array(W * S);
    let pos = Array.from(pre).length;
    words2.forEach((w, i) => {
      const a = pos, b = pos + Array.from(w).length;
      pos = b + 1;
      const idx = [];
      offsets.forEach(([s, e], k) => {
        if (e > s && s < b && e > a) idx.push(k);
      });
      for (const k of idx) wavg[i * S + k] = 1 / Math.max(1, idx.length);
    });
    return { ettin_ids: ids, ettin_mask: ids.map(() => 1), wavg, W, S };
  }
  function wordIndex(sentIds, wordIds, nWords, spaceId = 16) {
    const joined = [];
    wordIds.forEach((p, i) => {
      joined.push(...p);
      if (i < wordIds.length - 1) joined.push(spaceId);
    });
    if (joined.length === sentIds.length && joined.every((x, i) => x === sentIds[i])) {
      const w2 = [];
      wordIds.forEach((p, i) => {
        for (const _ of p) w2.push(i);
        if (i < wordIds.length - 1) w2.push(i);
      });
      return w2;
    }
    const nw = Math.max(1, sentIds.filter((x) => x === spaceId).length + 1);
    let k = 0;
    const w = [];
    for (const x of sentIds) {
      w.push(Math.min(Math.max(nWords, 1) - 1, Math.floor(k * nWords / nw)));
      if (x === spaceId) k++;
    }
    return w;
  }

  // frontend_entry.mjs
  function termsReplacer(table) {
    const terms = readTable(table);
    if (terms.size === 0) return (t) => t;
    const esc = (x) => x.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const forms = [...terms.keys()].sort((a, b) => b.length - a.length).map((f) => f.startsWith(".") ? esc(f) : `(?<![\\p{L}\\p{N}_])${esc(f)}`);
    const pattern = new RegExp(`(?:${forms.join("|")})(?![\\p{L}\\p{N}_])`, "gu");
    const lookUp = (t) => t.replace(pattern, (f, at, whole) => {
      const spoken = terms.get(f) ?? f;
      return f.startsWith(".") && at > 0 && /[\p{L}\p{N}]/u.test(whole[at - 1]) ? ` ${spoken}` : spoken;
    });
    const CAMEL = /(?<![\p{L}\p{N}_./])[a-z]+(?:[A-Z][a-z0-9]*)+(?![\p{L}\p{N}_/]|\.\p{L})/gu;
    const split = (t) => t.replace(CAMEL, (w) => w.replace(/(?<=[a-z0-9])(?=[A-Z])/g, " "));
    return (t) => lookUp(split(lookUp(t)));
  }
  function b64(s) {
    const A = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/", m = new Uint8Array(256);
    for (let i = 0; i < 64; i++) m[A.charCodeAt(i)] = i;
    s = s.replace(/=+$/, "");
    const out = new Uint8Array(s.length * 3 >> 2);
    let o = 0;
    for (let i = 0; i < s.length; i += 4) {
      const n = m[s.charCodeAt(i)] << 18 | m[s.charCodeAt(i + 1)] << 12 | m[s.charCodeAt(i + 2)] << 6 | m[s.charCodeAt(i + 3)];
      out[o++] = n >> 16;
      if (o < out.length) out[o++] = n >> 8 & 255;
      if (o < out.length) out[o++] = n & 255;
    }
    return out.buffer;
  }
  var fes = {};
  function make(lexicon, override, terms, g2pB64, gb) {
    const lex = readTable(lexicon);
    readTable(override, lex);
    const g2p = gb ? new BritishG2p(lex, new G2pModel(b64(g2pB64))) : new G2p(lex, new G2pModel(b64(g2pB64)));
    const norm2 = gb ? britishNormalizer(lex) : normalizer(lex), applyTerms = termsReplacer(terms);
    return { g2p, normalize: (t) => norm2(applyTerms(t)) };
  }
  globalThis.kaleFrontEnd = {
    init(lexicon, override, terms, g2pB64) {
      fes["en-us"] = make(lexicon, override, terms, g2pB64, false);
      return true;
    },
    initGb(lexicon, override, terms, g2pB64) {
      fes["en-gb"] = make(lexicon, override, terms, g2pB64, true);
      return true;
    },
    // = frontend/release-loader.mjs
    initEttin(tokenizerJson) {
      fes.ettin = new EttinTokenizer(JSON.parse(tokenizerJson));
      return true;
    },
    // Ettin inputs for one sentence (kale-tts-1-tiny's Ettin text side): sub-tokens with the previous sentence as context, word averages,
    // and the word index for the sentence's sound ids (sentIds, per word: the same front end's ids for that word)
    ettin(text, prev, sentIdsJson, wordIdsJson) {
      const e = ettinInputs(fes.ettin, text, prev), words2 = text.split(/\s+/).filter(Boolean);
      return JSON.stringify({ ids: e.ettin_ids, W: e.W, S: e.S, wavg: Array.from(e.wavg), widx: wordIndex(JSON.parse(sentIdsJson), JSON.parse(wordIdsJson), words2.length) });
    },
    ipaWords(text, accent = "en-us") {
      const fe = fes[accent] ?? fes["en-us"];
      return JSON.stringify(text.split(/\s+/).filter(Boolean).map((w) => fe.g2p.phonemes(w)));
    },
    run(text, accent = "en-us") {
      const fe = fes[accent] ?? fes["en-us"];
      return JSON.stringify(sentences(text, fe.normalize).map((s) => ({ text: s, ipa: fe.g2p.phonemes(s) })));
    }
  };
})();
