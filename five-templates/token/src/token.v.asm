Debug compilation of: five-templates/token/src/token.v
Source code:
// Token Implementation
// @test-params

account Mint {
    authority: pubkey;
    freeze_authority: pubkey;
    supply: u64;
    decimals: u8;
    name: string;
    symbol: string;
    uri: string;
}

account TokenAccount {
    owner: pubkey;
    mint: pubkey;
    balance: u64;
    is_frozen: bool;
    delegated_amount: u64;
    delegate: pubkey;
    initialized: bool;
}

pub init_mint(
    mint_account: Mint @mut @init(payer=authority, space=256) @signer,
    authority: account @mut @signer,
    freeze_authority: pubkey,
    decimals: u8,
    name: string,
    symbol: string,
    uri: string
) -> pubkey {
    require(decimals <= 20);
    mint_account.authority = authority.key;
    mint_account.freeze_authority = freeze_authority;
    mint_account.supply = 0;
    mint_account.decimals = decimals;
    mint_account.name = name;
    mint_account.symbol = symbol;
    mint_account.uri = uri;
    return mint_account.key;
}

pub init_token_account(
    token_account: TokenAccount @mut @init(payer=owner, space=192) @signer,
    owner: account @signer,
    mint: pubkey
) -> pubkey {
    token_account.owner = owner.key;
    token_account.mint = mint;
    token_account.balance = 0;
    token_account.is_frozen = false;
    token_account.delegated_amount = 0;
    token_account.delegate = 0;
    token_account.initialized = true;
    return token_account.key;
}

pub mint_to(
    mint_state: Mint @mut,
    destination_account: TokenAccount @mut,
    mint_authority: account @signer,
    amount: u64
) {
    require(mint_state.authority == mint_authority.key);
    require(destination_account.mint == mint_state.key);
    require(!destination_account.is_frozen);
    require(amount > 0);
    mint_state.supply = mint_state.supply + amount;
    destination_account.balance = destination_account.balance + amount;
}

pub transfer(
    source_account: TokenAccount @mut,
    destination_account: TokenAccount @mut,
    owner: account @signer,
    amount: u64
) {
    require(source_account.owner == owner.key);
    require(source_account.balance >= amount);
    require(source_account.mint == destination_account.mint);
    require(!source_account.is_frozen);
    require(!destination_account.is_frozen);
    require(amount > 0);
    source_account.balance = source_account.balance - amount;
    destination_account.balance = destination_account.balance + amount;
}

pub transfer_from(
    source_account: TokenAccount @mut,
    destination_account: TokenAccount @mut,
    authority: account @signer,
    amount: u64
) {
    let is_owner = source_account.owner == authority.key;
    if (!is_owner) {
        require(source_account.delegate == authority.key);
        require(source_account.delegated_amount >= amount);
    }
    require(source_account.balance >= amount);
    require(source_account.mint == destination_account.mint);
    require(!source_account.is_frozen);
    require(!destination_account.is_frozen);
    require(amount > 0);
    if (!is_owner) {
        source_account.delegated_amount = source_account.delegated_amount - amount;
    }
    source_account.balance = source_account.balance - amount;
    destination_account.balance = destination_account.balance + amount;
}

pub approve(
    source_account: TokenAccount @mut,
    owner: account @signer,
    delegate: pubkey,
    amount: u64
) {
    require(source_account.owner == owner.key);
    source_account.delegate = delegate;
    source_account.delegated_amount = amount;
}

pub revoke(
    source_account: TokenAccount @mut,
    owner: account @signer
) {
    require(source_account.owner == owner.key);
    source_account.delegate = 0;
    source_account.delegated_amount = 0;
}

pub burn(
    mint_state: Mint @mut,
    source_account: TokenAccount @mut,
    owner: account @signer,
    amount: u64
) {
    require(source_account.owner == owner.key);
    require(source_account.balance >= amount);
    require(source_account.mint == mint_state.key);
    require(!source_account.is_frozen);
    require(amount > 0);
    mint_state.supply = mint_state.supply - amount;
    source_account.balance = source_account.balance - amount;
}

pub freeze_account(
    mint_state: Mint,
    account_to_freeze: TokenAccount @mut,
    freeze_authority: account @signer
) {
    require(mint_state.freeze_authority == freeze_authority.key);
    require(account_to_freeze.mint == mint_state.key);
    account_to_freeze.is_frozen = true;
}

pub thaw_account(
    mint_state: Mint,
    account_to_thaw: TokenAccount @mut,
    freeze_authority: account @signer
) {
    require(mint_state.freeze_authority == freeze_authority.key);
    require(account_to_thaw.mint == mint_state.key);
    account_to_thaw.is_frozen = false;
}

pub set_mint_authority(
    mint_state: Mint @mut,
    current_authority: account @signer,
    new_authority: pubkey
) {
    require(mint_state.authority == current_authority.key);
    mint_state.authority = new_authority;
}

pub set_freeze_authority(
    mint_state: Mint @mut,
    current_freeze_authority: account @signer,
    new_freeze_authority: pubkey
) {
    require(mint_state.freeze_authority == current_freeze_authority.key);
    mint_state.freeze_authority = new_freeze_authority;
}

pub disable_mint(
    mint_state: Mint @mut,
    current_authority: account @signer
) {
    require(mint_state.authority == current_authority.key);
    mint_state.authority = 0;
}

pub disable_freeze(
    mint_state: Mint @mut,
    current_freeze_authority: account @signer
) {
    require(mint_state.freeze_authority == current_freeze_authority.key);
    mint_state.freeze_authority = 0;
}

============================================================

1. TOKENIZATION
--------------------
✓ Tokenization successful! Found 976 tokens:
  0: Account
  1: Identifier("Mint")
  2: LeftBrace
  3: Identifier("authority")
  4: Colon
  5: Type("pubkey")
  6: Semicolon
  7: Identifier("freeze_authority")
  8: Colon
  9: Type("pubkey")
  10: Semicolon
  11: Identifier("supply")
  12: Colon
  13: Type("u64")
  14: Semicolon
  15: Identifier("decimals")
  16: Colon
  17: Type("u8")
  18: Semicolon
  19: Identifier("name")
  20: Colon
  21: Type("string")
  22: Semicolon
  23: Identifier("symbol")
  24: Colon
  25: Type("string")
  26: Semicolon
  27: Identifier("uri")
  28: Colon
  29: Type("string")
  30: Semicolon
  31: RightBrace
  32: Account
  33: Identifier("TokenAccount")
  34: LeftBrace
  35: Identifier("owner")
  36: Colon
  37: Type("pubkey")
  38: Semicolon
  39: Identifier("mint")
  40: Colon
  41: Type("pubkey")
  42: Semicolon
  43: Identifier("balance")
  44: Colon
  45: Type("u64")
  46: Semicolon
  47: Identifier("is_frozen")
  48: Colon
  49: Type("bool")
  50: Semicolon
  51: Identifier("delegated_amount")
  52: Colon
  53: Type("u64")
  54: Semicolon
  55: Identifier("delegate")
  56: Colon
  57: Type("pubkey")
  58: Semicolon
  59: Identifier("initialized")
  60: Colon
  61: Type("bool")
  62: Semicolon
  63: RightBrace
  64: Pub
  65: Identifier("init_mint")
  66: LeftParen
  67: Identifier("mint_account")
  68: Colon
  69: Identifier("Mint")
  70: AtMut
  71: AtInit
  72: LeftParen
  73: Identifier("payer")
  74: Assign
  75: Identifier("authority")
  76: Comma
  77: Identifier("space")
  78: Assign
  79: NumberLiteral(256)
  80: RightParen
  81: AtSigner
  82: Comma
  83: Identifier("authority")
  84: Colon
  85: Account
  86: AtMut
  87: AtSigner
  88: Comma
  89: Identifier("freeze_authority")
  90: Colon
  91: Type("pubkey")
  92: Comma
  93: Identifier("decimals")
  94: Colon
  95: Type("u8")
  96: Comma
  97: Identifier("name")
  98: Colon
  99: Type("string")
  100: Comma
  101: Identifier("symbol")
  102: Colon
  103: Type("string")
  104: Comma
  105: Identifier("uri")
  106: Colon
  107: Type("string")
  108: RightParen
  109: Arrow
  110: Type("pubkey")
  111: LeftBrace
  112: Require
  113: LeftParen
  114: Identifier("decimals")
  115: LessEqual
  116: NumberLiteral(20)
  117: RightParen
  118: Semicolon
  119: Identifier("mint_account")
  120: Dot
  121: Identifier("authority")
  122: Assign
  123: Identifier("authority")
  124: Dot
  125: Identifier("key")
  126: Semicolon
  127: Identifier("mint_account")
  128: Dot
  129: Identifier("freeze_authority")
  130: Assign
  131: Identifier("freeze_authority")
  132: Semicolon
  133: Identifier("mint_account")
  134: Dot
  135: Identifier("supply")
  136: Assign
  137: NumberLiteral(0)
  138: Semicolon
  139: Identifier("mint_account")
  140: Dot
  141: Identifier("decimals")
  142: Assign
  143: Identifier("decimals")
  144: Semicolon
  145: Identifier("mint_account")
  146: Dot
  147: Identifier("name")
  148: Assign
  149: Identifier("name")
  150: Semicolon
  151: Identifier("mint_account")
  152: Dot
  153: Identifier("symbol")
  154: Assign
  155: Identifier("symbol")
  156: Semicolon
  157: Identifier("mint_account")
  158: Dot
  159: Identifier("uri")
  160: Assign
  161: Identifier("uri")
  162: Semicolon
  163: Return
  164: Identifier("mint_account")
  165: Dot
  166: Identifier("key")
  167: Semicolon
  168: RightBrace
  169: Pub
  170: Identifier("init_token_account")
  171: LeftParen
  172: Identifier("token_account")
  173: Colon
  174: Identifier("TokenAccount")
  175: AtMut
  176: AtInit
  177: LeftParen
  178: Identifier("payer")
  179: Assign
  180: Identifier("owner")
  181: Comma
  182: Identifier("space")
  183: Assign
  184: NumberLiteral(192)
  185: RightParen
  186: AtSigner
  187: Comma
  188: Identifier("owner")
  189: Colon
  190: Account
  191: AtSigner
  192: Comma
  193: Identifier("mint")
  194: Colon
  195: Type("pubkey")
  196: RightParen
  197: Arrow
  198: Type("pubkey")
  199: LeftBrace
  200: Identifier("token_account")
  201: Dot
  202: Identifier("owner")
  203: Assign
  204: Identifier("owner")
  205: Dot
  206: Identifier("key")
  207: Semicolon
  208: Identifier("token_account")
  209: Dot
  210: Identifier("mint")
  211: Assign
  212: Identifier("mint")
  213: Semicolon
  214: Identifier("token_account")
  215: Dot
  216: Identifier("balance")
  217: Assign
  218: NumberLiteral(0)
  219: Semicolon
  220: Identifier("token_account")
  221: Dot
  222: Identifier("is_frozen")
  223: Assign
  224: False
  225: Semicolon
  226: Identifier("token_account")
  227: Dot
  228: Identifier("delegated_amount")
  229: Assign
  230: NumberLiteral(0)
  231: Semicolon
  232: Identifier("token_account")
  233: Dot
  234: Identifier("delegate")
  235: Assign
  236: NumberLiteral(0)
  237: Semicolon
  238: Identifier("token_account")
  239: Dot
  240: Identifier("initialized")
  241: Assign
  242: True
  243: Semicolon
  244: Return
  245: Identifier("token_account")
  246: Dot
  247: Identifier("key")
  248: Semicolon
  249: RightBrace
  250: Pub
  251: Identifier("mint_to")
  252: LeftParen
  253: Identifier("mint_state")
  254: Colon
  255: Identifier("Mint")
  256: AtMut
  257: Comma
  258: Identifier("destination_account")
  259: Colon
  260: Identifier("TokenAccount")
  261: AtMut
  262: Comma
  263: Identifier("mint_authority")
  264: Colon
  265: Account
  266: AtSigner
  267: Comma
  268: Identifier("amount")
  269: Colon
  270: Type("u64")
  271: RightParen
  272: LeftBrace
  273: Require
  274: LeftParen
  275: Identifier("mint_state")
  276: Dot
  277: Identifier("authority")
  278: Equal
  279: Identifier("mint_authority")
  280: Dot
  281: Identifier("key")
  282: RightParen
  283: Semicolon
  284: Require
  285: LeftParen
  286: Identifier("destination_account")
  287: Dot
  288: Identifier("mint")
  289: Equal
  290: Identifier("mint_state")
  291: Dot
  292: Identifier("key")
  293: RightParen
  294: Semicolon
  295: Require
  296: LeftParen
  297: Bang
  298: Identifier("destination_account")
  299: Dot
  300: Identifier("is_frozen")
  301: RightParen
  302: Semicolon
  303: Require
  304: LeftParen
  305: Identifier("amount")
  306: GT
  307: NumberLiteral(0)
  308: RightParen
  309: Semicolon
  310: Identifier("mint_state")
  311: Dot
  312: Identifier("supply")
  313: Assign
  314: Identifier("mint_state")
  315: Dot
  316: Identifier("supply")
  317: Plus
  318: Identifier("amount")
  319: Semicolon
  320: Identifier("destination_account")
  321: Dot
  322: Identifier("balance")
  323: Assign
  324: Identifier("destination_account")
  325: Dot
  326: Identifier("balance")
  327: Plus
  328: Identifier("amount")
  329: Semicolon
  330: RightBrace
  331: Pub
  332: Identifier("transfer")
  333: LeftParen
  334: Identifier("source_account")
  335: Colon
  336: Identifier("TokenAccount")
  337: AtMut
  338: Comma
  339: Identifier("destination_account")
  340: Colon
  341: Identifier("TokenAccount")
  342: AtMut
  343: Comma
  344: Identifier("owner")
  345: Colon
  346: Account
  347: AtSigner
  348: Comma
  349: Identifier("amount")
  350: Colon
  351: Type("u64")
  352: RightParen
  353: LeftBrace
  354: Require
  355: LeftParen
  356: Identifier("source_account")
  357: Dot
  358: Identifier("owner")
  359: Equal
  360: Identifier("owner")
  361: Dot
  362: Identifier("key")
  363: RightParen
  364: Semicolon
  365: Require
  366: LeftParen
  367: Identifier("source_account")
  368: Dot
  369: Identifier("balance")
  370: GreaterEqual
  371: Identifier("amount")
  372: RightParen
  373: Semicolon
  374: Require
  375: LeftParen
  376: Identifier("source_account")
  377: Dot
  378: Identifier("mint")
  379: Equal
  380: Identifier("destination_account")
  381: Dot
  382: Identifier("mint")
  383: RightParen
  384: Semicolon
  385: Require
  386: LeftParen
  387: Bang
  388: Identifier("source_account")
  389: Dot
  390: Identifier("is_frozen")
  391: RightParen
  392: Semicolon
  393: Require
  394: LeftParen
  395: Bang
  396: Identifier("destination_account")
  397: Dot
  398: Identifier("is_frozen")
  399: RightParen
  400: Semicolon
  401: Require
  402: LeftParen
  403: Identifier("amount")
  404: GT
  405: NumberLiteral(0)
  406: RightParen
  407: Semicolon
  408: Identifier("source_account")
  409: Dot
  410: Identifier("balance")
  411: Assign
  412: Identifier("source_account")
  413: Dot
  414: Identifier("balance")
  415: Minus
  416: Identifier("amount")
  417: Semicolon
  418: Identifier("destination_account")
  419: Dot
  420: Identifier("balance")
  421: Assign
  422: Identifier("destination_account")
  423: Dot
  424: Identifier("balance")
  425: Plus
  426: Identifier("amount")
  427: Semicolon
  428: RightBrace
  429: Pub
  430: Identifier("transfer_from")
  431: LeftParen
  432: Identifier("source_account")
  433: Colon
  434: Identifier("TokenAccount")
  435: AtMut
  436: Comma
  437: Identifier("destination_account")
  438: Colon
  439: Identifier("TokenAccount")
  440: AtMut
  441: Comma
  442: Identifier("authority")
  443: Colon
  444: Account
  445: AtSigner
  446: Comma
  447: Identifier("amount")
  448: Colon
  449: Type("u64")
  450: RightParen
  451: LeftBrace
  452: Let
  453: Identifier("is_owner")
  454: Assign
  455: Identifier("source_account")
  456: Dot
  457: Identifier("owner")
  458: Equal
  459: Identifier("authority")
  460: Dot
  461: Identifier("key")
  462: Semicolon
  463: If
  464: LeftParen
  465: Bang
  466: Identifier("is_owner")
  467: RightParen
  468: LeftBrace
  469: Require
  470: LeftParen
  471: Identifier("source_account")
  472: Dot
  473: Identifier("delegate")
  474: Equal
  475: Identifier("authority")
  476: Dot
  477: Identifier("key")
  478: RightParen
  479: Semicolon
  480: Require
  481: LeftParen
  482: Identifier("source_account")
  483: Dot
  484: Identifier("delegated_amount")
  485: GreaterEqual
  486: Identifier("amount")
  487: RightParen
  488: Semicolon
  489: RightBrace
  490: Require
  491: LeftParen
  492: Identifier("source_account")
  493: Dot
  494: Identifier("balance")
  495: GreaterEqual
  496: Identifier("amount")
  497: RightParen
  498: Semicolon
  499: Require
  500: LeftParen
  501: Identifier("source_account")
  502: Dot
  503: Identifier("mint")
  504: Equal
  505: Identifier("destination_account")
  506: Dot
  507: Identifier("mint")
  508: RightParen
  509: Semicolon
  510: Require
  511: LeftParen
  512: Bang
  513: Identifier("source_account")
  514: Dot
  515: Identifier("is_frozen")
  516: RightParen
  517: Semicolon
  518: Require
  519: LeftParen
  520: Bang
  521: Identifier("destination_account")
  522: Dot
  523: Identifier("is_frozen")
  524: RightParen
  525: Semicolon
  526: Require
  527: LeftParen
  528: Identifier("amount")
  529: GT
  530: NumberLiteral(0)
  531: RightParen
  532: Semicolon
  533: If
  534: LeftParen
  535: Bang
  536: Identifier("is_owner")
  537: RightParen
  538: LeftBrace
  539: Identifier("source_account")
  540: Dot
  541: Identifier("delegated_amount")
  542: Assign
  543: Identifier("source_account")
  544: Dot
  545: Identifier("delegated_amount")
  546: Minus
  547: Identifier("amount")
  548: Semicolon
  549: RightBrace
  550: Identifier("source_account")
  551: Dot
  552: Identifier("balance")
  553: Assign
  554: Identifier("source_account")
  555: Dot
  556: Identifier("balance")
  557: Minus
  558: Identifier("amount")
  559: Semicolon
  560: Identifier("destination_account")
  561: Dot
  562: Identifier("balance")
  563: Assign
  564: Identifier("destination_account")
  565: Dot
  566: Identifier("balance")
  567: Plus
  568: Identifier("amount")
  569: Semicolon
  570: RightBrace
  571: Pub
  572: Identifier("approve")
  573: LeftParen
  574: Identifier("source_account")
  575: Colon
  576: Identifier("TokenAccount")
  577: AtMut
  578: Comma
  579: Identifier("owner")
  580: Colon
  581: Account
  582: AtSigner
  583: Comma
  584: Identifier("delegate")
  585: Colon
  586: Type("pubkey")
  587: Comma
  588: Identifier("amount")
  589: Colon
  590: Type("u64")
  591: RightParen
  592: LeftBrace
  593: Require
  594: LeftParen
  595: Identifier("source_account")
  596: Dot
  597: Identifier("owner")
  598: Equal
  599: Identifier("owner")
  600: Dot
  601: Identifier("key")
  602: RightParen
  603: Semicolon
  604: Identifier("source_account")
  605: Dot
  606: Identifier("delegate")
  607: Assign
  608: Identifier("delegate")
  609: Semicolon
  610: Identifier("source_account")
  611: Dot
  612: Identifier("delegated_amount")
  613: Assign
  614: Identifier("amount")
  615: Semicolon
  616: RightBrace
  617: Pub
  618: Identifier("revoke")
  619: LeftParen
  620: Identifier("source_account")
  621: Colon
  622: Identifier("TokenAccount")
  623: AtMut
  624: Comma
  625: Identifier("owner")
  626: Colon
  627: Account
  628: AtSigner
  629: RightParen
  630: LeftBrace
  631: Require
  632: LeftParen
  633: Identifier("source_account")
  634: Dot
  635: Identifier("owner")
  636: Equal
  637: Identifier("owner")
  638: Dot
  639: Identifier("key")
  640: RightParen
  641: Semicolon
  642: Identifier("source_account")
  643: Dot
  644: Identifier("delegate")
  645: Assign
  646: NumberLiteral(0)
  647: Semicolon
  648: Identifier("source_account")
  649: Dot
  650: Identifier("delegated_amount")
  651: Assign
  652: NumberLiteral(0)
  653: Semicolon
  654: RightBrace
  655: Pub
  656: Identifier("burn")
  657: LeftParen
  658: Identifier("mint_state")
  659: Colon
  660: Identifier("Mint")
  661: AtMut
  662: Comma
  663: Identifier("source_account")
  664: Colon
  665: Identifier("TokenAccount")
  666: AtMut
  667: Comma
  668: Identifier("owner")
  669: Colon
  670: Account
  671: AtSigner
  672: Comma
  673: Identifier("amount")
  674: Colon
  675: Type("u64")
  676: RightParen
  677: LeftBrace
  678: Require
  679: LeftParen
  680: Identifier("source_account")
  681: Dot
  682: Identifier("owner")
  683: Equal
  684: Identifier("owner")
  685: Dot
  686: Identifier("key")
  687: RightParen
  688: Semicolon
  689: Require
  690: LeftParen
  691: Identifier("source_account")
  692: Dot
  693: Identifier("balance")
  694: GreaterEqual
  695: Identifier("amount")
  696: RightParen
  697: Semicolon
  698: Require
  699: LeftParen
  700: Identifier("source_account")
  701: Dot
  702: Identifier("mint")
  703: Equal
  704: Identifier("mint_state")
  705: Dot
  706: Identifier("key")
  707: RightParen
  708: Semicolon
  709: Require
  710: LeftParen
  711: Bang
  712: Identifier("source_account")
  713: Dot
  714: Identifier("is_frozen")
  715: RightParen
  716: Semicolon
  717: Require
  718: LeftParen
  719: Identifier("amount")
  720: GT
  721: NumberLiteral(0)
  722: RightParen
  723: Semicolon
  724: Identifier("mint_state")
  725: Dot
  726: Identifier("supply")
  727: Assign
  728: Identifier("mint_state")
  729: Dot
  730: Identifier("supply")
  731: Minus
  732: Identifier("amount")
  733: Semicolon
  734: Identifier("source_account")
  735: Dot
  736: Identifier("balance")
  737: Assign
  738: Identifier("source_account")
  739: Dot
  740: Identifier("balance")
  741: Minus
  742: Identifier("amount")
  743: Semicolon
  744: RightBrace
  745: Pub
  746: Identifier("freeze_account")
  747: LeftParen
  748: Identifier("mint_state")
  749: Colon
  750: Identifier("Mint")
  751: Comma
  752: Identifier("account_to_freeze")
  753: Colon
  754: Identifier("TokenAccount")
  755: AtMut
  756: Comma
  757: Identifier("freeze_authority")
  758: Colon
  759: Account
  760: AtSigner
  761: RightParen
  762: LeftBrace
  763: Require
  764: LeftParen
  765: Identifier("mint_state")
  766: Dot
  767: Identifier("freeze_authority")
  768: Equal
  769: Identifier("freeze_authority")
  770: Dot
  771: Identifier("key")
  772: RightParen
  773: Semicolon
  774: Require
  775: LeftParen
  776: Identifier("account_to_freeze")
  777: Dot
  778: Identifier("mint")
  779: Equal
  780: Identifier("mint_state")
  781: Dot
  782: Identifier("key")
  783: RightParen
  784: Semicolon
  785: Identifier("account_to_freeze")
  786: Dot
  787: Identifier("is_frozen")
  788: Assign
  789: True
  790: Semicolon
  791: RightBrace
  792: Pub
  793: Identifier("thaw_account")
  794: LeftParen
  795: Identifier("mint_state")
  796: Colon
  797: Identifier("Mint")
  798: Comma
  799: Identifier("account_to_thaw")
  800: Colon
  801: Identifier("TokenAccount")
  802: AtMut
  803: Comma
  804: Identifier("freeze_authority")
  805: Colon
  806: Account
  807: AtSigner
  808: RightParen
  809: LeftBrace
  810: Require
  811: LeftParen
  812: Identifier("mint_state")
  813: Dot
  814: Identifier("freeze_authority")
  815: Equal
  816: Identifier("freeze_authority")
  817: Dot
  818: Identifier("key")
  819: RightParen
  820: Semicolon
  821: Require
  822: LeftParen
  823: Identifier("account_to_thaw")
  824: Dot
  825: Identifier("mint")
  826: Equal
  827: Identifier("mint_state")
  828: Dot
  829: Identifier("key")
  830: RightParen
  831: Semicolon
  832: Identifier("account_to_thaw")
  833: Dot
  834: Identifier("is_frozen")
  835: Assign
  836: False
  837: Semicolon
  838: RightBrace
  839: Pub
  840: Identifier("set_mint_authority")
  841: LeftParen
  842: Identifier("mint_state")
  843: Colon
  844: Identifier("Mint")
  845: AtMut
  846: Comma
  847: Identifier("current_authority")
  848: Colon
  849: Account
  850: AtSigner
  851: Comma
  852: Identifier("new_authority")
  853: Colon
  854: Type("pubkey")
  855: RightParen
  856: LeftBrace
  857: Require
  858: LeftParen
  859: Identifier("mint_state")
  860: Dot
  861: Identifier("authority")
  862: Equal
  863: Identifier("current_authority")
  864: Dot
  865: Identifier("key")
  866: RightParen
  867: Semicolon
  868: Identifier("mint_state")
  869: Dot
  870: Identifier("authority")
  871: Assign
  872: Identifier("new_authority")
  873: Semicolon
  874: RightBrace
  875: Pub
  876: Identifier("set_freeze_authority")
  877: LeftParen
  878: Identifier("mint_state")
  879: Colon
  880: Identifier("Mint")
  881: AtMut
  882: Comma
  883: Identifier("current_freeze_authority")
  884: Colon
  885: Account
  886: AtSigner
  887: Comma
  888: Identifier("new_freeze_authority")
  889: Colon
  890: Type("pubkey")
  891: RightParen
  892: LeftBrace
  893: Require
  894: LeftParen
  895: Identifier("mint_state")
  896: Dot
  897: Identifier("freeze_authority")
  898: Equal
  899: Identifier("current_freeze_authority")
  900: Dot
  901: Identifier("key")
  902: RightParen
  903: Semicolon
  904: Identifier("mint_state")
  905: Dot
  906: Identifier("freeze_authority")
  907: Assign
  908: Identifier("new_freeze_authority")
  909: Semicolon
  910: RightBrace
  911: Pub
  912: Identifier("disable_mint")
  913: LeftParen
  914: Identifier("mint_state")
  915: Colon
  916: Identifier("Mint")
  917: AtMut
  918: Comma
  919: Identifier("current_authority")
  920: Colon
  921: Account
  922: AtSigner
  923: RightParen
  924: LeftBrace
  925: Require
  926: LeftParen
  927: Identifier("mint_state")
  928: Dot
  929: Identifier("authority")
  930: Equal
  931: Identifier("current_authority")
  932: Dot
  933: Identifier("key")
  934: RightParen
  935: Semicolon
  936: Identifier("mint_state")
  937: Dot
  938: Identifier("authority")
  939: Assign
  940: NumberLiteral(0)
  941: Semicolon
  942: RightBrace
  943: Pub
  944: Identifier("disable_freeze")
  945: LeftParen
  946: Identifier("mint_state")
  947: Colon
  948: Identifier("Mint")
  949: AtMut
  950: Comma
  951: Identifier("current_freeze_authority")
  952: Colon
  953: Account
  954: AtSigner
  955: RightParen
  956: LeftBrace
  957: Require
  958: LeftParen
  959: Identifier("mint_state")
  960: Dot
  961: Identifier("freeze_authority")
  962: Equal
  963: Identifier("current_freeze_authority")
  964: Dot
  965: Identifier("key")
  966: RightParen
  967: Semicolon
  968: Identifier("mint_state")
  969: Dot
  970: Identifier("freeze_authority")
  971: Assign
  972: NumberLiteral(0)
  973: Semicolon
  974: RightBrace
  975: Eof

2. PARSING
--------------------
OLIVIA_WATERMARK: DslBytecodeGenerator::generate finished
OLIVIA_WATERMARK: DslBytecodeGenerator::generate finished
OLIVIA_WATERMARK: DslBytecodeGenerator::generate finished
OLIVIA_WATERMARK: DslBytecodeGenerator::generate finished
OLIVIA_WATERMARK: DslBytecodeGenerator::generate finished
OLIVIA_WATERMARK: DslBytecodeGenerator::generate finished
OLIVIA_WATERMARK: DslBytecodeGenerator::generate finished
OLIVIA_WATERMARK: DslBytecodeGenerator::generate finished
OLIVIA_WATERMARK: DslBytecodeGenerator::generate finished
OLIVIA_WATERMARK: DslBytecodeGenerator::generate finished
OLIVIA_WATERMARK: DslBytecodeGenerator::generate finished
OLIVIA_WATERMARK: DslBytecodeGenerator::generate finished
OLIVIA_WATERMARK: DslBytecodeGenerator::generate finished
OLIVIA_WATERMARK: DslBytecodeGenerator::generate finished
✓ Parsing successful!
  AST: Program {
    program_name: "Module",
    field_definitions: [],
    instruction_definitions: [
        InstructionDefinition {
            name: "init_mint",
            parameters: [
                InstructionParameter {
                    name: "mint_account",
                    param_type: Named(
                        "Mint",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                        Attribute {
                            name: "init",
                            args: [],
                        },
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: true,
                    init_config: Some(
                        InitConfig {
                            seeds: None,
                            bump: None,
                            space: Some(
                                256,
                            ),
                            payer: Some(
                                "authority",
                            ),
                        },
                    ),
                },
                InstructionParameter {
                    name: "authority",
                    param_type: Account,
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "freeze_authority",
                    param_type: Primitive(
                        "pubkey",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "decimals",
                    param_type: Primitive(
                        "u8",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "name",
                    param_type: Primitive(
                        "string",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "symbol",
                    param_type: Primitive(
                        "string",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "uri",
                    param_type: Primitive(
                        "string",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
            ],
            return_type: Some(
                Primitive(
                    "pubkey",
                ),
            ),
            body: Block {
                statements: [
                    RequireStatement {
                        condition: MethodCall {
                            object: Identifier(
                                "decimals",
                            ),
                            method: "lte",
                            args: [
                                Literal(
                                    U64(
                                        20,
                                    ),
                                ),
                            ],
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "mint_account",
                        ),
                        field: "authority",
                        value: FieldAccess {
                            object: Identifier(
                                "authority",
                            ),
                            field: "key",
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "mint_account",
                        ),
                        field: "freeze_authority",
                        value: Identifier(
                            "freeze_authority",
                        ),
                    },
                    FieldAssignment {
                        object: Identifier(
                            "mint_account",
                        ),
                        field: "supply",
                        value: Literal(
                            U64(
                                0,
                            ),
                        ),
                    },
                    FieldAssignment {
                        object: Identifier(
                            "mint_account",
                        ),
                        field: "decimals",
                        value: Identifier(
                            "decimals",
                        ),
                    },
                    FieldAssignment {
                        object: Identifier(
                            "mint_account",
                        ),
                        field: "name",
                        value: Identifier(
                            "name",
                        ),
                    },
                    FieldAssignment {
                        object: Identifier(
                            "mint_account",
                        ),
                        field: "symbol",
                        value: Identifier(
                            "symbol",
                        ),
                    },
                    FieldAssignment {
                        object: Identifier(
                            "mint_account",
                        ),
                        field: "uri",
                        value: Identifier(
                            "uri",
                        ),
                    },
                    ReturnStatement {
                        value: Some(
                            FieldAccess {
                                object: Identifier(
                                    "mint_account",
                                ),
                                field: "key",
                            },
                        ),
                    },
                ],
                kind: Regular,
            },
            visibility: Public,
            is_public: true,
        },
        InstructionDefinition {
            name: "init_token_account",
            parameters: [
                InstructionParameter {
                    name: "token_account",
                    param_type: Named(
                        "TokenAccount",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                        Attribute {
                            name: "init",
                            args: [],
                        },
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: true,
                    init_config: Some(
                        InitConfig {
                            seeds: None,
                            bump: None,
                            space: Some(
                                192,
                            ),
                            payer: Some(
                                "owner",
                            ),
                        },
                    ),
                },
                InstructionParameter {
                    name: "owner",
                    param_type: Account,
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "mint",
                    param_type: Primitive(
                        "pubkey",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
            ],
            return_type: Some(
                Primitive(
                    "pubkey",
                ),
            ),
            body: Block {
                statements: [
                    FieldAssignment {
                        object: Identifier(
                            "token_account",
                        ),
                        field: "owner",
                        value: FieldAccess {
                            object: Identifier(
                                "owner",
                            ),
                            field: "key",
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "token_account",
                        ),
                        field: "mint",
                        value: Identifier(
                            "mint",
                        ),
                    },
                    FieldAssignment {
                        object: Identifier(
                            "token_account",
                        ),
                        field: "balance",
                        value: Literal(
                            U64(
                                0,
                            ),
                        ),
                    },
                    FieldAssignment {
                        object: Identifier(
                            "token_account",
                        ),
                        field: "is_frozen",
                        value: Literal(
                            Bool(
                                false,
                            ),
                        ),
                    },
                    FieldAssignment {
                        object: Identifier(
                            "token_account",
                        ),
                        field: "delegated_amount",
                        value: Literal(
                            U64(
                                0,
                            ),
                        ),
                    },
                    FieldAssignment {
                        object: Identifier(
                            "token_account",
                        ),
                        field: "delegate",
                        value: Literal(
                            U64(
                                0,
                            ),
                        ),
                    },
                    FieldAssignment {
                        object: Identifier(
                            "token_account",
                        ),
                        field: "initialized",
                        value: Literal(
                            Bool(
                                true,
                            ),
                        ),
                    },
                    ReturnStatement {
                        value: Some(
                            FieldAccess {
                                object: Identifier(
                                    "token_account",
                                ),
                                field: "key",
                            },
                        ),
                    },
                ],
                kind: Regular,
            },
            visibility: Public,
            is_public: true,
        },
        InstructionDefinition {
            name: "mint_to",
            parameters: [
                InstructionParameter {
                    name: "mint_state",
                    param_type: Named(
                        "Mint",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "destination_account",
                    param_type: Named(
                        "TokenAccount",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "mint_authority",
                    param_type: Account,
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "amount",
                    param_type: Primitive(
                        "u64",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
            ],
            return_type: None,
            body: Block {
                statements: [
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "mint_state",
                                ),
                                field: "authority",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "mint_authority",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "destination_account",
                                ),
                                field: "mint",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "mint_state",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    RequireStatement {
                        condition: UnaryExpression {
                            operator: "not",
                            operand: FieldAccess {
                                object: Identifier(
                                    "destination_account",
                                ),
                                field: "is_frozen",
                            },
                        },
                    },
                    RequireStatement {
                        condition: MethodCall {
                            object: Identifier(
                                "amount",
                            ),
                            method: "gt",
                            args: [
                                Literal(
                                    U64(
                                        0,
                                    ),
                                ),
                            ],
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "mint_state",
                        ),
                        field: "supply",
                        value: BinaryExpression {
                            operator: "+",
                            left: FieldAccess {
                                object: Identifier(
                                    "mint_state",
                                ),
                                field: "supply",
                            },
                            right: Identifier(
                                "amount",
                            ),
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "destination_account",
                        ),
                        field: "balance",
                        value: BinaryExpression {
                            operator: "+",
                            left: FieldAccess {
                                object: Identifier(
                                    "destination_account",
                                ),
                                field: "balance",
                            },
                            right: Identifier(
                                "amount",
                            ),
                        },
                    },
                ],
                kind: Regular,
            },
            visibility: Public,
            is_public: true,
        },
        InstructionDefinition {
            name: "transfer",
            parameters: [
                InstructionParameter {
                    name: "source_account",
                    param_type: Named(
                        "TokenAccount",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "destination_account",
                    param_type: Named(
                        "TokenAccount",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "owner",
                    param_type: Account,
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "amount",
                    param_type: Primitive(
                        "u64",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
            ],
            return_type: None,
            body: Block {
                statements: [
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "owner",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "owner",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "balance",
                            },
                            method: "gte",
                            args: [
                                Identifier(
                                    "amount",
                                ),
                            ],
                        },
                    },
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "mint",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "destination_account",
                                    ),
                                    field: "mint",
                                },
                            ],
                        },
                    },
                    RequireStatement {
                        condition: UnaryExpression {
                            operator: "not",
                            operand: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "is_frozen",
                            },
                        },
                    },
                    RequireStatement {
                        condition: UnaryExpression {
                            operator: "not",
                            operand: FieldAccess {
                                object: Identifier(
                                    "destination_account",
                                ),
                                field: "is_frozen",
                            },
                        },
                    },
                    RequireStatement {
                        condition: MethodCall {
                            object: Identifier(
                                "amount",
                            ),
                            method: "gt",
                            args: [
                                Literal(
                                    U64(
                                        0,
                                    ),
                                ),
                            ],
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "source_account",
                        ),
                        field: "balance",
                        value: BinaryExpression {
                            operator: "-",
                            left: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "balance",
                            },
                            right: Identifier(
                                "amount",
                            ),
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "destination_account",
                        ),
                        field: "balance",
                        value: BinaryExpression {
                            operator: "+",
                            left: FieldAccess {
                                object: Identifier(
                                    "destination_account",
                                ),
                                field: "balance",
                            },
                            right: Identifier(
                                "amount",
                            ),
                        },
                    },
                ],
                kind: Regular,
            },
            visibility: Public,
            is_public: true,
        },
        InstructionDefinition {
            name: "transfer_from",
            parameters: [
                InstructionParameter {
                    name: "source_account",
                    param_type: Named(
                        "TokenAccount",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "destination_account",
                    param_type: Named(
                        "TokenAccount",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "authority",
                    param_type: Account,
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "amount",
                    param_type: Primitive(
                        "u64",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
            ],
            return_type: None,
            body: Block {
                statements: [
                    LetStatement {
                        name: "is_owner",
                        type_annotation: None,
                        is_mutable: false,
                        value: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "owner",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "authority",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    IfStatement {
                        condition: UnaryExpression {
                            operator: "not",
                            operand: Identifier(
                                "is_owner",
                            ),
                        },
                        then_branch: Block {
                            statements: [
                                RequireStatement {
                                    condition: MethodCall {
                                        object: FieldAccess {
                                            object: Identifier(
                                                "source_account",
                                            ),
                                            field: "delegate",
                                        },
                                        method: "eq",
                                        args: [
                                            FieldAccess {
                                                object: Identifier(
                                                    "authority",
                                                ),
                                                field: "key",
                                            },
                                        ],
                                    },
                                },
                                RequireStatement {
                                    condition: MethodCall {
                                        object: FieldAccess {
                                            object: Identifier(
                                                "source_account",
                                            ),
                                            field: "delegated_amount",
                                        },
                                        method: "gte",
                                        args: [
                                            Identifier(
                                                "amount",
                                            ),
                                        ],
                                    },
                                },
                            ],
                            kind: Regular,
                        },
                        else_branch: None,
                    },
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "balance",
                            },
                            method: "gte",
                            args: [
                                Identifier(
                                    "amount",
                                ),
                            ],
                        },
                    },
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "mint",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "destination_account",
                                    ),
                                    field: "mint",
                                },
                            ],
                        },
                    },
                    RequireStatement {
                        condition: UnaryExpression {
                            operator: "not",
                            operand: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "is_frozen",
                            },
                        },
                    },
                    RequireStatement {
                        condition: UnaryExpression {
                            operator: "not",
                            operand: FieldAccess {
                                object: Identifier(
                                    "destination_account",
                                ),
                                field: "is_frozen",
                            },
                        },
                    },
                    RequireStatement {
                        condition: MethodCall {
                            object: Identifier(
                                "amount",
                            ),
                            method: "gt",
                            args: [
                                Literal(
                                    U64(
                                        0,
                                    ),
                                ),
                            ],
                        },
                    },
                    IfStatement {
                        condition: UnaryExpression {
                            operator: "not",
                            operand: Identifier(
                                "is_owner",
                            ),
                        },
                        then_branch: Block {
                            statements: [
                                FieldAssignment {
                                    object: Identifier(
                                        "source_account",
                                    ),
                                    field: "delegated_amount",
                                    value: BinaryExpression {
                                        operator: "-",
                                        left: FieldAccess {
                                            object: Identifier(
                                                "source_account",
                                            ),
                                            field: "delegated_amount",
                                        },
                                        right: Identifier(
                                            "amount",
                                        ),
                                    },
                                },
                            ],
                            kind: Regular,
                        },
                        else_branch: None,
                    },
                    FieldAssignment {
                        object: Identifier(
                            "source_account",
                        ),
                        field: "balance",
                        value: BinaryExpression {
                            operator: "-",
                            left: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "balance",
                            },
                            right: Identifier(
                                "amount",
                            ),
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "destination_account",
                        ),
                        field: "balance",
                        value: BinaryExpression {
                            operator: "+",
                            left: FieldAccess {
                                object: Identifier(
                                    "destination_account",
                                ),
                                field: "balance",
                            },
                            right: Identifier(
                                "amount",
                            ),
                        },
                    },
                ],
                kind: Regular,
            },
            visibility: Public,
            is_public: true,
        },
        InstructionDefinition {
            name: "approve",
            parameters: [
                InstructionParameter {
                    name: "source_account",
                    param_type: Named(
                        "TokenAccount",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "owner",
                    param_type: Account,
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "delegate",
                    param_type: Primitive(
                        "pubkey",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "amount",
                    param_type: Primitive(
                        "u64",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
            ],
            return_type: None,
            body: Block {
                statements: [
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "owner",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "owner",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "source_account",
                        ),
                        field: "delegate",
                        value: Identifier(
                            "delegate",
                        ),
                    },
                    FieldAssignment {
                        object: Identifier(
                            "source_account",
                        ),
                        field: "delegated_amount",
                        value: Identifier(
                            "amount",
                        ),
                    },
                ],
                kind: Regular,
            },
            visibility: Public,
            is_public: true,
        },
        InstructionDefinition {
            name: "revoke",
            parameters: [
                InstructionParameter {
                    name: "source_account",
                    param_type: Named(
                        "TokenAccount",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "owner",
                    param_type: Account,
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
            ],
            return_type: None,
            body: Block {
                statements: [
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "owner",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "owner",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "source_account",
                        ),
                        field: "delegate",
                        value: Literal(
                            U64(
                                0,
                            ),
                        ),
                    },
                    FieldAssignment {
                        object: Identifier(
                            "source_account",
                        ),
                        field: "delegated_amount",
                        value: Literal(
                            U64(
                                0,
                            ),
                        ),
                    },
                ],
                kind: Regular,
            },
            visibility: Public,
            is_public: true,
        },
        InstructionDefinition {
            name: "burn",
            parameters: [
                InstructionParameter {
                    name: "mint_state",
                    param_type: Named(
                        "Mint",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "source_account",
                    param_type: Named(
                        "TokenAccount",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "owner",
                    param_type: Account,
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "amount",
                    param_type: Primitive(
                        "u64",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
            ],
            return_type: None,
            body: Block {
                statements: [
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "owner",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "owner",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "balance",
                            },
                            method: "gte",
                            args: [
                                Identifier(
                                    "amount",
                                ),
                            ],
                        },
                    },
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "mint",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "mint_state",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    RequireStatement {
                        condition: UnaryExpression {
                            operator: "not",
                            operand: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "is_frozen",
                            },
                        },
                    },
                    RequireStatement {
                        condition: MethodCall {
                            object: Identifier(
                                "amount",
                            ),
                            method: "gt",
                            args: [
                                Literal(
                                    U64(
                                        0,
                                    ),
                                ),
                            ],
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "mint_state",
                        ),
                        field: "supply",
                        value: BinaryExpression {
                            operator: "-",
                            left: FieldAccess {
                                object: Identifier(
                                    "mint_state",
                                ),
                                field: "supply",
                            },
                            right: Identifier(
                                "amount",
                            ),
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "source_account",
                        ),
                        field: "balance",
                        value: BinaryExpression {
                            operator: "-",
                            left: FieldAccess {
                                object: Identifier(
                                    "source_account",
                                ),
                                field: "balance",
                            },
                            right: Identifier(
                                "amount",
                            ),
                        },
                    },
                ],
                kind: Regular,
            },
            visibility: Public,
            is_public: true,
        },
        InstructionDefinition {
            name: "freeze_account",
            parameters: [
                InstructionParameter {
                    name: "mint_state",
                    param_type: Named(
                        "Mint",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "account_to_freeze",
                    param_type: Named(
                        "TokenAccount",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "freeze_authority",
                    param_type: Account,
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
            ],
            return_type: None,
            body: Block {
                statements: [
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "mint_state",
                                ),
                                field: "freeze_authority",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "freeze_authority",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "account_to_freeze",
                                ),
                                field: "mint",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "mint_state",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "account_to_freeze",
                        ),
                        field: "is_frozen",
                        value: Literal(
                            Bool(
                                true,
                            ),
                        ),
                    },
                ],
                kind: Regular,
            },
            visibility: Public,
            is_public: true,
        },
        InstructionDefinition {
            name: "thaw_account",
            parameters: [
                InstructionParameter {
                    name: "mint_state",
                    param_type: Named(
                        "Mint",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "account_to_thaw",
                    param_type: Named(
                        "TokenAccount",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "freeze_authority",
                    param_type: Account,
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
            ],
            return_type: None,
            body: Block {
                statements: [
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "mint_state",
                                ),
                                field: "freeze_authority",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "freeze_authority",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "account_to_thaw",
                                ),
                                field: "mint",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "mint_state",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "account_to_thaw",
                        ),
                        field: "is_frozen",
                        value: Literal(
                            Bool(
                                false,
                            ),
                        ),
                    },
                ],
                kind: Regular,
            },
            visibility: Public,
            is_public: true,
        },
        InstructionDefinition {
            name: "set_mint_authority",
            parameters: [
                InstructionParameter {
                    name: "mint_state",
                    param_type: Named(
                        "Mint",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "current_authority",
                    param_type: Account,
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "new_authority",
                    param_type: Primitive(
                        "pubkey",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
            ],
            return_type: None,
            body: Block {
                statements: [
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "mint_state",
                                ),
                                field: "authority",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "current_authority",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "mint_state",
                        ),
                        field: "authority",
                        value: Identifier(
                            "new_authority",
                        ),
                    },
                ],
                kind: Regular,
            },
            visibility: Public,
            is_public: true,
        },
        InstructionDefinition {
            name: "set_freeze_authority",
            parameters: [
                InstructionParameter {
                    name: "mint_state",
                    param_type: Named(
                        "Mint",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "current_freeze_authority",
                    param_type: Account,
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "new_freeze_authority",
                    param_type: Primitive(
                        "pubkey",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [],
                    is_init: false,
                    init_config: None,
                },
            ],
            return_type: None,
            body: Block {
                statements: [
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "mint_state",
                                ),
                                field: "freeze_authority",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "current_freeze_authority",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "mint_state",
                        ),
                        field: "freeze_authority",
                        value: Identifier(
                            "new_freeze_authority",
                        ),
                    },
                ],
                kind: Regular,
            },
            visibility: Public,
            is_public: true,
        },
        InstructionDefinition {
            name: "disable_mint",
            parameters: [
                InstructionParameter {
                    name: "mint_state",
                    param_type: Named(
                        "Mint",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "current_authority",
                    param_type: Account,
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
            ],
            return_type: None,
            body: Block {
                statements: [
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "mint_state",
                                ),
                                field: "authority",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "current_authority",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "mint_state",
                        ),
                        field: "authority",
                        value: Literal(
                            U64(
                                0,
                            ),
                        ),
                    },
                ],
                kind: Regular,
            },
            visibility: Public,
            is_public: true,
        },
        InstructionDefinition {
            name: "disable_freeze",
            parameters: [
                InstructionParameter {
                    name: "mint_state",
                    param_type: Named(
                        "Mint",
                    ),
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "mut",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
                InstructionParameter {
                    name: "current_freeze_authority",
                    param_type: Account,
                    is_optional: false,
                    default_value: None,
                    attributes: [
                        Attribute {
                            name: "signer",
                            args: [],
                        },
                    ],
                    is_init: false,
                    init_config: None,
                },
            ],
            return_type: None,
            body: Block {
                statements: [
                    RequireStatement {
                        condition: MethodCall {
                            object: FieldAccess {
                                object: Identifier(
                                    "mint_state",
                                ),
                                field: "freeze_authority",
                            },
                            method: "eq",
                            args: [
                                FieldAccess {
                                    object: Identifier(
                                        "current_freeze_authority",
                                    ),
                                    field: "key",
                                },
                            ],
                        },
                    },
                    FieldAssignment {
                        object: Identifier(
                            "mint_state",
                        ),
                        field: "freeze_authority",
                        value: Literal(
                            U64(
                                0,
                            ),
                        ),
                    },
                ],
                kind: Regular,
            },
            visibility: Public,
            is_public: true,
        },
    ],
    event_definitions: [],
    account_definitions: [
        AccountDefinition {
            name: "Mint",
            fields: [
                StructField {
                    name: "authority",
                    field_type: Primitive(
                        "pubkey",
                    ),
                    is_mutable: false,
                    is_optional: false,
                },
                StructField {
                    name: "freeze_authority",
                    field_type: Primitive(
                        "pubkey",
                    ),
                    is_mutable: false,
                    is_optional: false,
                },
                StructField {
                    name: "supply",
                    field_type: Primitive(
                        "u64",
                    ),
                    is_mutable: false,
                    is_optional: false,
                },
                StructField {
                    name: "decimals",
                    field_type: Primitive(
                        "u8",
                    ),
                    is_mutable: false,
                    is_optional: false,
                },
                StructField {
                    name: "name",
                    field_type: Primitive(
                        "string",
                    ),
                    is_mutable: false,
                    is_optional: false,
                },
                StructField {
                    name: "symbol",
                    field_type: Primitive(
                        "string",
                    ),
                    is_mutable: false,
                    is_optional: false,
                },
                StructField {
                    name: "uri",
                    field_type: Primitive(
                        "string",
                    ),
                    is_mutable: false,
                    is_optional: false,
                },
            ],
            visibility: Internal,
        },
        AccountDefinition {
            name: "TokenAccount",
            fields: [
                StructField {
                    name: "owner",
                    field_type: Primitive(
                        "pubkey",
                    ),
                    is_mutable: false,
                    is_optional: false,
                },
                StructField {
                    name: "mint",
                    field_type: Primitive(
                        "pubkey",
                    ),
                    is_mutable: false,
                    is_optional: false,
                },
                StructField {
                    name: "balance",
                    field_type: Primitive(
                        "u64",
                    ),
                    is_mutable: false,
                    is_optional: false,
                },
                StructField {
                    name: "is_frozen",
                    field_type: Primitive(
                        "bool",
                    ),
                    is_mutable: false,
                    is_optional: false,
                },
                StructField {
                    name: "delegated_amount",
                    field_type: Primitive(
                        "u64",
                    ),
                    is_mutable: false,
                    is_optional: false,
                },
                StructField {
                    name: "delegate",
                    field_type: Primitive(
                        "pubkey",
                    ),
                    is_mutable: false,
                    is_optional: false,
                },
                StructField {
                    name: "initialized",
                    field_type: Primitive(
                        "bool",
                    ),
                    is_mutable: false,
                    is_optional: false,
                },
            ],
            visibility: Internal,
        },
    ],
    interface_definitions: [],
    import_statements: [],
    init_block: None,
    constraints_block: None,
}

3. TYPE CHECKING
--------------------
✓ Type checking successful!
  Type checking passed

4. BYTECODE GENERATION
--------------------
DEBUG: has_callable_functions - instruction_definitions.len() = 14
DEBUG: has_callable_functions - init_block.is_some() = false
DEBUG: instruction_definition[0] = init_mint (public: true)
DEBUG: instruction_definition[1] = init_token_account (public: true)
DEBUG: instruction_definition[2] = mint_to (public: true)
DEBUG: instruction_definition[3] = transfer (public: true)
DEBUG: instruction_definition[4] = transfer_from (public: true)
DEBUG: instruction_definition[5] = approve (public: true)
DEBUG: instruction_definition[6] = revoke (public: true)
DEBUG: instruction_definition[7] = burn (public: true)
DEBUG: instruction_definition[8] = freeze_account (public: true)
DEBUG: instruction_definition[9] = thaw_account (public: true)
DEBUG: instruction_definition[10] = set_mint_authority (public: true)
DEBUG: instruction_definition[11] = set_freeze_authority (public: true)
DEBUG: instruction_definition[12] = disable_mint (public: true)
DEBUG: instruction_definition[13] = disable_freeze (public: true)
DEBUG: has_callable_functions returning true (has_functions: true, init_block: false)
DEBUG: has_callable_functions = true
DEBUG: Function ordering validated - 14 public functions (indices 0..13), 0 private functions (indices 14..13)
DEBUG: Collected 14 public functions, 14 total functions for optimized header
DEBUG: Using OptimizedHeader V2 with explicit public_count and total_count
DEBUG: Emitting function name metadata (debug info enabled)
DEBUG: Taking function coordination path - has_functions = true
DEBUG: Processed field definitions, symbol table has 0 entries
AccountSystem: Processing account definition 'Mint'
AccountSystem: Adding field 'authority' type 'pubkey' at offset 0 (size: 32)
AccountSystem: Adding field 'freeze_authority' type 'pubkey' at offset 32 (size: 32)
AccountSystem: Adding field 'supply' type 'u64' at offset 64 (size: 8)
AccountSystem: Adding field 'decimals' type 'u8' at offset 72 (size: 1)
AccountSystem: Adding field 'name' type 'string' at offset 73 (size: 32)
AccountSystem: Adding field 'symbol' type 'string' at offset 105 (size: 32)
AccountSystem: Adding field 'uri' type 'string' at offset 137 (size: 32)
AccountSystem: Registered account type 'Mint' with 7 fields (total size: 169)
AccountSystem: Processing account definition 'TokenAccount'
AccountSystem: Adding field 'owner' type 'pubkey' at offset 0 (size: 32)
AccountSystem: Adding field 'mint' type 'pubkey' at offset 32 (size: 32)
AccountSystem: Adding field 'balance' type 'u64' at offset 64 (size: 8)
AccountSystem: Adding field 'is_frozen' type 'bool' at offset 72 (size: 1)
AccountSystem: Adding field 'delegated_amount' type 'u64' at offset 73 (size: 8)
AccountSystem: Adding field 'delegate' type 'pubkey' at offset 81 (size: 32)
AccountSystem: Adding field 'initialized' type 'bool' at offset 113 (size: 1)
AccountSystem: Registered account type 'TokenAccount' with 7 fields (total size: 114)
DEBUG: Processed account definitions in function coordination path
AccountSystem: Processing account definition 'Mint'
AccountSystem: Adding field 'authority' type 'pubkey' at offset 0 (size: 32)
AccountSystem: Adding field 'freeze_authority' type 'pubkey' at offset 32 (size: 32)
AccountSystem: Adding field 'supply' type 'u64' at offset 64 (size: 8)
AccountSystem: Adding field 'decimals' type 'u8' at offset 72 (size: 1)
AccountSystem: Adding field 'name' type 'string' at offset 73 (size: 32)
AccountSystem: Adding field 'symbol' type 'string' at offset 105 (size: 32)
AccountSystem: Adding field 'uri' type 'string' at offset 137 (size: 32)
AccountSystem: Registered account type 'Mint' with 7 fields (total size: 169)
AccountSystem: Processing account definition 'TokenAccount'
AccountSystem: Adding field 'owner' type 'pubkey' at offset 0 (size: 32)
AccountSystem: Adding field 'mint' type 'pubkey' at offset 32 (size: 32)
AccountSystem: Adding field 'balance' type 'u64' at offset 64 (size: 8)
AccountSystem: Adding field 'is_frozen' type 'bool' at offset 72 (size: 1)
AccountSystem: Adding field 'delegated_amount' type 'u64' at offset 73 (size: 8)
AccountSystem: Adding field 'delegate' type 'pubkey' at offset 81 (size: 32)
AccountSystem: Adding field 'initialized' type 'bool' at offset 113 (size: 1)
AccountSystem: Registered account type 'TokenAccount' with 7 fields (total size: 114)
BytecodeGenerator: Attempting to process imports from AST node...
BytecodeGenerator: AST Node is Program. Found 0 import statements
DEBUG: has_callable_functions - instruction_definitions.len() = 14
DEBUG: has_callable_functions - init_block.is_some() = false
DEBUG: instruction_definition[0] = init_mint (public: true)
DEBUG: instruction_definition[1] = init_token_account (public: true)
DEBUG: instruction_definition[2] = mint_to (public: true)
DEBUG: instruction_definition[3] = transfer (public: true)
DEBUG: instruction_definition[4] = transfer_from (public: true)
DEBUG: instruction_definition[5] = approve (public: true)
DEBUG: instruction_definition[6] = revoke (public: true)
DEBUG: instruction_definition[7] = burn (public: true)
DEBUG: instruction_definition[8] = freeze_account (public: true)
DEBUG: instruction_definition[9] = thaw_account (public: true)
DEBUG: instruction_definition[10] = set_mint_authority (public: true)
DEBUG: instruction_definition[11] = set_freeze_authority (public: true)
DEBUG: instruction_definition[12] = disable_mint (public: true)
DEBUG: instruction_definition[13] = disable_freeze (public: true)
DEBUG: has_callable_functions returning true (has_functions: true, init_block: false)
DEBUG: Function ordering validated - 14 public functions (indices 0..13), 0 private functions (indices 14..13)
🔒 Five DSL: Running security analysis...
✅ Five DSL: Security analysis passed - no violations detected
AccountSystem: Processing account definition 'Mint'
AccountSystem: Adding field 'authority' type 'pubkey' at offset 0 (size: 32)
AccountSystem: Adding field 'freeze_authority' type 'pubkey' at offset 32 (size: 32)
AccountSystem: Adding field 'supply' type 'u64' at offset 64 (size: 8)
AccountSystem: Adding field 'decimals' type 'u8' at offset 72 (size: 1)
AccountSystem: Adding field 'name' type 'string' at offset 73 (size: 32)
AccountSystem: Adding field 'symbol' type 'string' at offset 105 (size: 32)
AccountSystem: Adding field 'uri' type 'string' at offset 137 (size: 32)
AccountSystem: Registered account type 'Mint' with 7 fields (total size: 169)
AccountSystem: Processing account definition 'TokenAccount'
AccountSystem: Adding field 'owner' type 'pubkey' at offset 0 (size: 32)
AccountSystem: Adding field 'mint' type 'pubkey' at offset 32 (size: 32)
AccountSystem: Adding field 'balance' type 'u64' at offset 64 (size: 8)
AccountSystem: Adding field 'is_frozen' type 'bool' at offset 72 (size: 1)
AccountSystem: Adding field 'delegated_amount' type 'u64' at offset 73 (size: 8)
AccountSystem: Adding field 'delegate' type 'pubkey' at offset 81 (size: 32)
AccountSystem: Adding field 'initialized' type 'bool' at offset 113 (size: 1)
AccountSystem: Registered account type 'TokenAccount' with 7 fields (total size: 114)
DEBUG: has_callable_functions - instruction_definitions.len() = 14
DEBUG: has_callable_functions - init_block.is_some() = false
DEBUG: instruction_definition[0] = init_mint (public: true)
DEBUG: instruction_definition[1] = init_token_account (public: true)
DEBUG: instruction_definition[2] = mint_to (public: true)
DEBUG: instruction_definition[3] = transfer (public: true)
DEBUG: instruction_definition[4] = transfer_from (public: true)
DEBUG: instruction_definition[5] = approve (public: true)
DEBUG: instruction_definition[6] = revoke (public: true)
DEBUG: instruction_definition[7] = burn (public: true)
DEBUG: instruction_definition[8] = freeze_account (public: true)
DEBUG: instruction_definition[9] = thaw_account (public: true)
DEBUG: instruction_definition[10] = set_mint_authority (public: true)
DEBUG: instruction_definition[11] = set_freeze_authority (public: true)
DEBUG: instruction_definition[12] = disable_mint (public: true)
DEBUG: instruction_definition[13] = disable_freeze (public: true)
DEBUG: has_callable_functions returning true (has_functions: true, init_block: false)
DEBUG: Generating function dispatch logic (Jump Table)
DEBUG: Processing instruction definition: revoke
DEBUG: About to generate function body for: revoke
DEBUG: generate_parameter_loading called with 2 parameters
DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
@@@ INIT_SEQUENCE_CHECK: param='source_account', is_init=false, index=0
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'source_account'
@@@ INIT_SEQUENCE_CHECK: param='owner', is_init=false, index=1
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'owner'
DEBUG: About to generate AST node for function: revoke
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'owner'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'owner'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'owner' at offset 0
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'owner' field 'key'
AST Generator: Found symbol 'owner' with type 'Account'
AST Generator: Using built-in property access for 'owner.key'
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
    FieldAssignment details: object=Identifier("source_account"), field="delegate"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Literal"
AST Generator: Calculating field offset for account 'TokenAccount' field 'delegate'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'delegate' at offset 81
DSL Compiler DEBUG: FieldAssignment for account 'source_account', mapped to index 1, field_offset 81
    FieldAssignment details: object=Identifier("source_account"), field="delegated_amount"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Literal"
AST Generator: Calculating field offset for account 'TokenAccount' field 'delegated_amount'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'delegated_amount' at offset 73
DSL Compiler DEBUG: FieldAssignment for account 'source_account', mapped to index 1, field_offset 73
DEBUG: Completed AST node generation for function: revoke
DEBUG: Emitting RETURN opcode for void function: revoke
DEBUG: Completed function body generation for: revoke
DEBUG: Processing instruction definition: disable_mint
DEBUG: About to generate function body for: disable_mint
DEBUG: generate_parameter_loading called with 2 parameters
DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
@@@ INIT_SEQUENCE_CHECK: param='mint_state', is_init=false, index=0
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'mint_state'
@@@ INIT_SEQUENCE_CHECK: param='current_authority', is_init=false, index=1
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'current_authority'
DEBUG: About to generate AST node for function: disable_mint
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_state' field 'authority'
AST Generator: Found symbol 'mint_state' with type 'Mint'
AST Generator: Calculating field offset for account 'Mint' field 'authority'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'authority' at offset 0
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'current_authority' field 'key'
AST Generator: Found symbol 'current_authority' with type 'Account'
AST Generator: Using built-in property access for 'current_authority.key'
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
    FieldAssignment details: object=Identifier("mint_state"), field="authority"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Literal"
AST Generator: Calculating field offset for account 'Mint' field 'authority'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'authority' at offset 0
DSL Compiler DEBUG: FieldAssignment for account 'mint_state', mapped to index 1, field_offset 0
DEBUG: Completed AST node generation for function: disable_mint
DEBUG: Emitting RETURN opcode for void function: disable_mint
DEBUG: Completed function body generation for: disable_mint
DEBUG: Processing instruction definition: disable_freeze
DEBUG: About to generate function body for: disable_freeze
DEBUG: generate_parameter_loading called with 2 parameters
DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
@@@ INIT_SEQUENCE_CHECK: param='mint_state', is_init=false, index=0
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'mint_state'
@@@ INIT_SEQUENCE_CHECK: param='current_freeze_authority', is_init=false, index=1
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'current_freeze_authority'
DEBUG: About to generate AST node for function: disable_freeze
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_state' field 'freeze_authority'
AST Generator: Found symbol 'mint_state' with type 'Mint'
AST Generator: Calculating field offset for account 'Mint' field 'freeze_authority'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'freeze_authority' at offset 32
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'current_freeze_authority' field 'key'
AST Generator: Found symbol 'current_freeze_authority' with type 'Account'
AST Generator: Using built-in property access for 'current_freeze_authority.key'
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
    FieldAssignment details: object=Identifier("mint_state"), field="freeze_authority"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Literal"
AST Generator: Calculating field offset for account 'Mint' field 'freeze_authority'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'freeze_authority' at offset 32
DSL Compiler DEBUG: FieldAssignment for account 'mint_state', mapped to index 1, field_offset 32
DEBUG: Completed AST node generation for function: disable_freeze
DEBUG: Emitting RETURN opcode for void function: disable_freeze
DEBUG: Completed function body generation for: disable_freeze
DEBUG: Processing instruction definition: init_token_account
DEBUG: About to generate function body for: init_token_account
DEBUG: generate_parameter_loading called with 3 parameters
DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
@@@ INIT_SEQUENCE_CHECK: param='token_account', is_init=true, index=0
@@@ INIT_SEQUENCE_PROCEED: Generating initialization for param 'token_account'
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
@@@ INIT_SEQUENCE_CHECK: param='owner', is_init=false, index=1
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'owner'
@@@ INIT_SEQUENCE_CHECK: param='mint', is_init=false, index=2
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'mint'
DEBUG: About to generate AST node for function: init_token_account
AST Generator DEBUG: Processing node: "Other"
    FieldAssignment details: object=Identifier("token_account"), field="owner"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'owner' field 'key'
AST Generator: Found symbol 'owner' with type 'Account'
AST Generator: Using built-in property access for 'owner.key'
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator: Calculating field offset for account 'TokenAccount' field 'owner'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'owner' at offset 0
DSL Compiler DEBUG: FieldAssignment for account 'token_account', mapped to index 1, field_offset 0
    FieldAssignment details: object=Identifier("token_account"), field="mint"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: mint
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 3 for parameter 'mint'
AST Generator: Calculating field offset for account 'TokenAccount' field 'mint'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'mint' at offset 32
DSL Compiler DEBUG: FieldAssignment for account 'token_account', mapped to index 1, field_offset 32
    FieldAssignment details: object=Identifier("token_account"), field="balance"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Literal"
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DSL Compiler DEBUG: FieldAssignment for account 'token_account', mapped to index 1, field_offset 64
    FieldAssignment details: object=Identifier("token_account"), field="is_frozen"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Literal"
AST Generator: Calculating field offset for account 'TokenAccount' field 'is_frozen'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'is_frozen' at offset 72
DSL Compiler DEBUG: FieldAssignment for account 'token_account', mapped to index 1, field_offset 72
    FieldAssignment details: object=Identifier("token_account"), field="delegated_amount"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Literal"
AST Generator: Calculating field offset for account 'TokenAccount' field 'delegated_amount'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'delegated_amount' at offset 73
DSL Compiler DEBUG: FieldAssignment for account 'token_account', mapped to index 1, field_offset 73
    FieldAssignment details: object=Identifier("token_account"), field="delegate"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Literal"
AST Generator: Calculating field offset for account 'TokenAccount' field 'delegate'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'delegate' at offset 81
DSL Compiler DEBUG: FieldAssignment for account 'token_account', mapped to index 1, field_offset 81
    FieldAssignment details: object=Identifier("token_account"), field="initialized"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Literal"
AST Generator: Calculating field offset for account 'TokenAccount' field 'initialized'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'initialized' at offset 113
DSL Compiler DEBUG: FieldAssignment for account 'token_account', mapped to index 1, field_offset 113
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'token_account' field 'key'
AST Generator: Found symbol 'token_account' with type 'TokenAccount'
AST Generator: Using built-in property access for 'token_account.key'
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: Completed AST node generation for function: init_token_account
DEBUG: Function init_token_account has return type, using RETURN_VALUE from explicit return statement
DEBUG: Completed function body generation for: init_token_account
DEBUG: Processing instruction definition: freeze_account
DEBUG: About to generate function body for: freeze_account
DEBUG: generate_parameter_loading called with 3 parameters
DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
DEBUG: account_index_from_param_index(2) with offset 1 -> 3
@@@ INIT_SEQUENCE_CHECK: param='mint_state', is_init=false, index=0
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'mint_state'
@@@ INIT_SEQUENCE_CHECK: param='account_to_freeze', is_init=false, index=1
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'account_to_freeze'
@@@ INIT_SEQUENCE_CHECK: param='freeze_authority', is_init=false, index=2
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'freeze_authority'
DEBUG: About to generate AST node for function: freeze_account
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_state' field 'freeze_authority'
AST Generator: Found symbol 'mint_state' with type 'Mint'
AST Generator: Calculating field offset for account 'Mint' field 'freeze_authority'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'freeze_authority' at offset 32
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'freeze_authority' field 'key'
AST Generator: Found symbol 'freeze_authority' with type 'Account'
AST Generator: Using built-in property access for 'freeze_authority.key'
DEBUG: account_index_from_param_index(2) with offset 1 -> 3
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'account_to_freeze' field 'mint'
AST Generator: Found symbol 'account_to_freeze' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'mint'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'mint' at offset 32
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_state' field 'key'
AST Generator: Found symbol 'mint_state' with type 'Mint'
AST Generator: Using built-in property access for 'mint_state.key'
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    FieldAssignment details: object=Identifier("account_to_freeze"), field="is_frozen"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Literal"
AST Generator: Calculating field offset for account 'TokenAccount' field 'is_frozen'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'is_frozen' at offset 72
DSL Compiler DEBUG: FieldAssignment for account 'account_to_freeze', mapped to index 2, field_offset 72
DEBUG: Completed AST node generation for function: freeze_account
DEBUG: Emitting RETURN opcode for void function: freeze_account
DEBUG: Completed function body generation for: freeze_account
DEBUG: Processing instruction definition: thaw_account
DEBUG: About to generate function body for: thaw_account
DEBUG: generate_parameter_loading called with 3 parameters
DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
DEBUG: account_index_from_param_index(2) with offset 1 -> 3
@@@ INIT_SEQUENCE_CHECK: param='mint_state', is_init=false, index=0
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'mint_state'
@@@ INIT_SEQUENCE_CHECK: param='account_to_thaw', is_init=false, index=1
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'account_to_thaw'
@@@ INIT_SEQUENCE_CHECK: param='freeze_authority', is_init=false, index=2
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'freeze_authority'
DEBUG: About to generate AST node for function: thaw_account
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_state' field 'freeze_authority'
AST Generator: Found symbol 'mint_state' with type 'Mint'
AST Generator: Calculating field offset for account 'Mint' field 'freeze_authority'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'freeze_authority' at offset 32
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'freeze_authority' field 'key'
AST Generator: Found symbol 'freeze_authority' with type 'Account'
AST Generator: Using built-in property access for 'freeze_authority.key'
DEBUG: account_index_from_param_index(2) with offset 1 -> 3
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'account_to_thaw' field 'mint'
AST Generator: Found symbol 'account_to_thaw' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'mint'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'mint' at offset 32
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_state' field 'key'
AST Generator: Found symbol 'mint_state' with type 'Mint'
AST Generator: Using built-in property access for 'mint_state.key'
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    FieldAssignment details: object=Identifier("account_to_thaw"), field="is_frozen"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Literal"
AST Generator: Calculating field offset for account 'TokenAccount' field 'is_frozen'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'is_frozen' at offset 72
DSL Compiler DEBUG: FieldAssignment for account 'account_to_thaw', mapped to index 2, field_offset 72
DEBUG: Completed AST node generation for function: thaw_account
DEBUG: Emitting RETURN opcode for void function: thaw_account
DEBUG: Completed function body generation for: thaw_account
DEBUG: Processing instruction definition: set_mint_authority
DEBUG: About to generate function body for: set_mint_authority
DEBUG: generate_parameter_loading called with 3 parameters
DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
@@@ INIT_SEQUENCE_CHECK: param='mint_state', is_init=false, index=0
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'mint_state'
@@@ INIT_SEQUENCE_CHECK: param='current_authority', is_init=false, index=1
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'current_authority'
@@@ INIT_SEQUENCE_CHECK: param='new_authority', is_init=false, index=2
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'new_authority'
DEBUG: About to generate AST node for function: set_mint_authority
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_state' field 'authority'
AST Generator: Found symbol 'mint_state' with type 'Mint'
AST Generator: Calculating field offset for account 'Mint' field 'authority'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'authority' at offset 0
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'current_authority' field 'key'
AST Generator: Found symbol 'current_authority' with type 'Account'
AST Generator: Using built-in property access for 'current_authority.key'
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
    FieldAssignment details: object=Identifier("mint_state"), field="authority"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: new_authority
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 3 for parameter 'new_authority'
AST Generator: Calculating field offset for account 'Mint' field 'authority'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'authority' at offset 0
DSL Compiler DEBUG: FieldAssignment for account 'mint_state', mapped to index 1, field_offset 0
DEBUG: Completed AST node generation for function: set_mint_authority
DEBUG: Emitting RETURN opcode for void function: set_mint_authority
DEBUG: Completed function body generation for: set_mint_authority
DEBUG: Processing instruction definition: set_freeze_authority
DEBUG: About to generate function body for: set_freeze_authority
DEBUG: generate_parameter_loading called with 3 parameters
DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
@@@ INIT_SEQUENCE_CHECK: param='mint_state', is_init=false, index=0
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'mint_state'
@@@ INIT_SEQUENCE_CHECK: param='current_freeze_authority', is_init=false, index=1
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'current_freeze_authority'
@@@ INIT_SEQUENCE_CHECK: param='new_freeze_authority', is_init=false, index=2
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'new_freeze_authority'
DEBUG: About to generate AST node for function: set_freeze_authority
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_state' field 'freeze_authority'
AST Generator: Found symbol 'mint_state' with type 'Mint'
AST Generator: Calculating field offset for account 'Mint' field 'freeze_authority'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'freeze_authority' at offset 32
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'current_freeze_authority' field 'key'
AST Generator: Found symbol 'current_freeze_authority' with type 'Account'
AST Generator: Using built-in property access for 'current_freeze_authority.key'
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
    FieldAssignment details: object=Identifier("mint_state"), field="freeze_authority"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: new_freeze_authority
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 3 for parameter 'new_freeze_authority'
AST Generator: Calculating field offset for account 'Mint' field 'freeze_authority'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'freeze_authority' at offset 32
DSL Compiler DEBUG: FieldAssignment for account 'mint_state', mapped to index 1, field_offset 32
DEBUG: Completed AST node generation for function: set_freeze_authority
DEBUG: Emitting RETURN opcode for void function: set_freeze_authority
DEBUG: Completed function body generation for: set_freeze_authority
DEBUG: Processing instruction definition: mint_to
DEBUG: About to generate function body for: mint_to
DEBUG: generate_parameter_loading called with 4 parameters
DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
DEBUG: account_index_from_param_index(2) with offset 1 -> 3
@@@ INIT_SEQUENCE_CHECK: param='mint_state', is_init=false, index=0
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'mint_state'
@@@ INIT_SEQUENCE_CHECK: param='destination_account', is_init=false, index=1
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'destination_account'
@@@ INIT_SEQUENCE_CHECK: param='mint_authority', is_init=false, index=2
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'mint_authority'
@@@ INIT_SEQUENCE_CHECK: param='amount', is_init=false, index=3
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'amount'
DEBUG: About to generate AST node for function: mint_to
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_state' field 'authority'
AST Generator: Found symbol 'mint_state' with type 'Mint'
AST Generator: Calculating field offset for account 'Mint' field 'authority'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'authority' at offset 0
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_authority' field 'key'
AST Generator: Found symbol 'mint_authority' with type 'Account'
AST Generator: Using built-in property access for 'mint_authority.key'
DEBUG: account_index_from_param_index(2) with offset 1 -> 3
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'destination_account' field 'mint'
AST Generator: Found symbol 'destination_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'mint'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'mint' at offset 32
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_state' field 'key'
AST Generator: Found symbol 'mint_state' with type 'Mint'
AST Generator: Using built-in property access for 'mint_state.key'
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'destination_account' field 'is_frozen'
AST Generator: Found symbol 'destination_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'is_frozen'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'is_frozen' at offset 72
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='gt'
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator DEBUG: Processing node: "Literal"
    FieldAssignment details: object=Identifier("mint_state"), field="supply"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_state' field 'supply'
AST Generator: Found symbol 'mint_state' with type 'Mint'
AST Generator: Calculating field offset for account 'Mint' field 'supply'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'supply' at offset 64
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator: Calculating field offset for account 'Mint' field 'supply'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'supply' at offset 64
DSL Compiler DEBUG: FieldAssignment for account 'mint_state', mapped to index 1, field_offset 64
    FieldAssignment details: object=Identifier("destination_account"), field="balance"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'destination_account' field 'balance'
AST Generator: Found symbol 'destination_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DSL Compiler DEBUG: FieldAssignment for account 'destination_account', mapped to index 2, field_offset 64
DEBUG: Completed AST node generation for function: mint_to
DEBUG: Emitting RETURN opcode for void function: mint_to
DEBUG: Completed function body generation for: mint_to
DEBUG: Processing instruction definition: transfer
DEBUG: About to generate function body for: transfer
DEBUG: generate_parameter_loading called with 4 parameters
DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
DEBUG: account_index_from_param_index(2) with offset 1 -> 3
@@@ INIT_SEQUENCE_CHECK: param='source_account', is_init=false, index=0
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'source_account'
@@@ INIT_SEQUENCE_CHECK: param='destination_account', is_init=false, index=1
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'destination_account'
@@@ INIT_SEQUENCE_CHECK: param='owner', is_init=false, index=2
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'owner'
@@@ INIT_SEQUENCE_CHECK: param='amount', is_init=false, index=3
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'amount'
DEBUG: About to generate AST node for function: transfer
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'owner'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'owner'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'owner' at offset 0
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'owner' field 'key'
AST Generator: Found symbol 'owner' with type 'Account'
AST Generator: Using built-in property access for 'owner.key'
DEBUG: account_index_from_param_index(2) with offset 1 -> 3
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='gte'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'balance'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'mint'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'mint'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'mint' at offset 32
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'destination_account' field 'mint'
AST Generator: Found symbol 'destination_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'mint'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'mint' at offset 32
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'is_frozen'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'is_frozen'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'is_frozen' at offset 72
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'destination_account' field 'is_frozen'
AST Generator: Found symbol 'destination_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'is_frozen'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'is_frozen' at offset 72
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='gt'
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator DEBUG: Processing node: "Literal"
    FieldAssignment details: object=Identifier("source_account"), field="balance"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'balance'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DSL Compiler DEBUG: FieldAssignment for account 'source_account', mapped to index 1, field_offset 64
    FieldAssignment details: object=Identifier("destination_account"), field="balance"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'destination_account' field 'balance'
AST Generator: Found symbol 'destination_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DSL Compiler DEBUG: FieldAssignment for account 'destination_account', mapped to index 2, field_offset 64
DEBUG: Completed AST node generation for function: transfer
DEBUG: Emitting RETURN opcode for void function: transfer
DEBUG: Completed function body generation for: transfer
DEBUG: Processing instruction definition: transfer_from
DEBUG: About to generate function body for: transfer_from
DEBUG: generate_parameter_loading called with 4 parameters
DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
DEBUG: account_index_from_param_index(2) with offset 1 -> 3
@@@ INIT_SEQUENCE_CHECK: param='source_account', is_init=false, index=0
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'source_account'
@@@ INIT_SEQUENCE_CHECK: param='destination_account', is_init=false, index=1
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'destination_account'
@@@ INIT_SEQUENCE_CHECK: param='authority', is_init=false, index=2
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'authority'
@@@ INIT_SEQUENCE_CHECK: param='amount', is_init=false, index=3
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'amount'
DEBUG: About to generate AST node for function: transfer_from
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'owner'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'owner'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'owner' at offset 0
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'authority' field 'key'
AST Generator: Found symbol 'authority' with type 'Account'
AST Generator: Using built-in property access for 'authority.key'
DEBUG: account_index_from_param_index(2) with offset 1 -> 3
DEBUG: Generated SET_LOCAL_0 (nibble immediate) for let statement 'is_owner'
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
    Identifier: is_owner
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated GET_LOCAL_0 (nibble immediate) for local variable 'is_owner'
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'delegate'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'delegate'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'delegate' at offset 81
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'authority' field 'key'
AST Generator: Found symbol 'authority' with type 'Account'
AST Generator: Using built-in property access for 'authority.key'
DEBUG: account_index_from_param_index(2) with offset 1 -> 3
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='gte'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'delegated_amount'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'delegated_amount'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'delegated_amount' at offset 73
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='gte'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'balance'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'mint'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'mint'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'mint' at offset 32
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'destination_account' field 'mint'
AST Generator: Found symbol 'destination_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'mint'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'mint' at offset 32
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'is_frozen'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'is_frozen'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'is_frozen' at offset 72
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'destination_account' field 'is_frozen'
AST Generator: Found symbol 'destination_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'is_frozen'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'is_frozen' at offset 72
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='gt'
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator DEBUG: Processing node: "Literal"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
    Identifier: is_owner
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated GET_LOCAL_0 (nibble immediate) for local variable 'is_owner'
AST Generator DEBUG: Processing node: "Other"
    FieldAssignment details: object=Identifier("source_account"), field="delegated_amount"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'delegated_amount'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'delegated_amount'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'delegated_amount' at offset 73
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'delegated_amount'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'delegated_amount' at offset 73
DSL Compiler DEBUG: FieldAssignment for account 'source_account', mapped to index 1, field_offset 73
    FieldAssignment details: object=Identifier("source_account"), field="balance"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'balance'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DSL Compiler DEBUG: FieldAssignment for account 'source_account', mapped to index 1, field_offset 64
    FieldAssignment details: object=Identifier("destination_account"), field="balance"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'destination_account' field 'balance'
AST Generator: Found symbol 'destination_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DSL Compiler DEBUG: FieldAssignment for account 'destination_account', mapped to index 2, field_offset 64
DEBUG: Completed AST node generation for function: transfer_from
DEBUG: Emitting RETURN opcode for void function: transfer_from
DEBUG: Completed function body generation for: transfer_from
DEBUG: Processing instruction definition: approve
DEBUG: About to generate function body for: approve
DEBUG: generate_parameter_loading called with 4 parameters
DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
@@@ INIT_SEQUENCE_CHECK: param='source_account', is_init=false, index=0
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'source_account'
@@@ INIT_SEQUENCE_CHECK: param='owner', is_init=false, index=1
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'owner'
@@@ INIT_SEQUENCE_CHECK: param='delegate', is_init=false, index=2
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'delegate'
@@@ INIT_SEQUENCE_CHECK: param='amount', is_init=false, index=3
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'amount'
DEBUG: About to generate AST node for function: approve
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'owner'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'owner'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'owner' at offset 0
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'owner' field 'key'
AST Generator: Found symbol 'owner' with type 'Account'
AST Generator: Using built-in property access for 'owner.key'
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
    FieldAssignment details: object=Identifier("source_account"), field="delegate"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: delegate
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 3 for parameter 'delegate'
AST Generator: Calculating field offset for account 'TokenAccount' field 'delegate'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'delegate' at offset 81
DSL Compiler DEBUG: FieldAssignment for account 'source_account', mapped to index 1, field_offset 81
    FieldAssignment details: object=Identifier("source_account"), field="delegated_amount"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'delegated_amount'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'delegated_amount' at offset 73
DSL Compiler DEBUG: FieldAssignment for account 'source_account', mapped to index 1, field_offset 73
DEBUG: Completed AST node generation for function: approve
DEBUG: Emitting RETURN opcode for void function: approve
DEBUG: Completed function body generation for: approve
DEBUG: Processing instruction definition: burn
DEBUG: About to generate function body for: burn
DEBUG: generate_parameter_loading called with 4 parameters
DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
DEBUG: account_index_from_param_index(2) with offset 1 -> 3
@@@ INIT_SEQUENCE_CHECK: param='mint_state', is_init=false, index=0
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'mint_state'
@@@ INIT_SEQUENCE_CHECK: param='source_account', is_init=false, index=1
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'source_account'
@@@ INIT_SEQUENCE_CHECK: param='owner', is_init=false, index=2
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'owner'
@@@ INIT_SEQUENCE_CHECK: param='amount', is_init=false, index=3
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'amount'
DEBUG: About to generate AST node for function: burn
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'owner'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'owner'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'owner' at offset 0
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'owner' field 'key'
AST Generator: Found symbol 'owner' with type 'Account'
AST Generator: Using built-in property access for 'owner.key'
DEBUG: account_index_from_param_index(2) with offset 1 -> 3
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='gte'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'balance'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='eq'
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'mint'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'mint'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'mint' at offset 32
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_state' field 'key'
AST Generator: Found symbol 'mint_state' with type 'Mint'
AST Generator: Using built-in property access for 'mint_state.key'
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'is_frozen'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'is_frozen'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'is_frozen' at offset 72
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='gt'
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator DEBUG: Processing node: "Literal"
    FieldAssignment details: object=Identifier("mint_state"), field="supply"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_state' field 'supply'
AST Generator: Found symbol 'mint_state' with type 'Mint'
AST Generator: Calculating field offset for account 'Mint' field 'supply'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'supply' at offset 64
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator: Calculating field offset for account 'Mint' field 'supply'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'supply' at offset 64
DSL Compiler DEBUG: FieldAssignment for account 'mint_state', mapped to index 1, field_offset 64
    FieldAssignment details: object=Identifier("source_account"), field="balance"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'source_account' field 'balance'
AST Generator: Found symbol 'source_account' with type 'TokenAccount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
    Identifier: amount
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'amount'
AST Generator: Calculating field offset for account 'TokenAccount' field 'balance'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'TokenAccount' with 7 fields
AST Generator: Account 'TokenAccount' has field 'delegate' at offset 81
AST Generator: Account 'TokenAccount' has field 'mint' at offset 32
AST Generator: Account 'TokenAccount' has field 'delegated_amount' at offset 73
AST Generator: Account 'TokenAccount' has field 'initialized' at offset 113
AST Generator: Account 'TokenAccount' has field 'is_frozen' at offset 72
AST Generator: Account 'TokenAccount' has field 'balance' at offset 64
AST Generator: Account 'TokenAccount' has field 'owner' at offset 0
AST Generator: Found field 'balance' at offset 64
DSL Compiler DEBUG: FieldAssignment for account 'source_account', mapped to index 2, field_offset 64
DEBUG: Completed AST node generation for function: burn
DEBUG: Emitting RETURN opcode for void function: burn
DEBUG: Completed function body generation for: burn
DEBUG: Processing instruction definition: init_mint
DEBUG: About to generate function body for: init_mint
DEBUG: generate_parameter_loading called with 7 parameters
DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
@@@ INIT_SEQUENCE_CHECK: param='mint_account', is_init=true, index=0
@@@ INIT_SEQUENCE_PROCEED: Generating initialization for param 'mint_account'
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
@@@ INIT_SEQUENCE_CHECK: param='authority', is_init=false, index=1
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'authority'
@@@ INIT_SEQUENCE_CHECK: param='freeze_authority', is_init=false, index=2
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'freeze_authority'
@@@ INIT_SEQUENCE_CHECK: param='decimals', is_init=false, index=3
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'decimals'
@@@ INIT_SEQUENCE_CHECK: param='name', is_init=false, index=4
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'name'
@@@ INIT_SEQUENCE_CHECK: param='symbol', is_init=false, index=5
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'symbol'
@@@ INIT_SEQUENCE_CHECK: param='uri', is_init=false, index=6
@@@ INIT_SEQUENCE_SKIP: is_init=false for param 'uri'
DEBUG: About to generate AST node for function: init_mint
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
DEBUG: generate_method_call method='lte'
    Identifier: decimals
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'decimals'
AST Generator DEBUG: Processing node: "Literal"
    FieldAssignment details: object=Identifier("mint_account"), field="authority"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'authority' field 'key'
AST Generator: Found symbol 'authority' with type 'Account'
AST Generator: Using built-in property access for 'authority.key'
DEBUG: account_index_from_param_index(1) with offset 1 -> 2
AST Generator: Calculating field offset for account 'Mint' field 'authority'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'authority' at offset 0
DSL Compiler DEBUG: FieldAssignment for account 'mint_account', mapped to index 1, field_offset 0
    FieldAssignment details: object=Identifier("mint_account"), field="freeze_authority"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: freeze_authority
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 3 for parameter 'freeze_authority'
AST Generator: Calculating field offset for account 'Mint' field 'freeze_authority'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'freeze_authority' at offset 32
DSL Compiler DEBUG: FieldAssignment for account 'mint_account', mapped to index 1, field_offset 32
    FieldAssignment details: object=Identifier("mint_account"), field="supply"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
AST Generator DEBUG: Processing node: "Literal"
AST Generator: Calculating field offset for account 'Mint' field 'supply'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'supply' at offset 64
DSL Compiler DEBUG: FieldAssignment for account 'mint_account', mapped to index 1, field_offset 64
    FieldAssignment details: object=Identifier("mint_account"), field="decimals"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: decimals
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 4 for parameter 'decimals'
AST Generator: Calculating field offset for account 'Mint' field 'decimals'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'decimals' at offset 72
DSL Compiler DEBUG: FieldAssignment for account 'mint_account', mapped to index 1, field_offset 72
    FieldAssignment details: object=Identifier("mint_account"), field="name"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: name
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 5 for parameter 'name'
AST Generator: Calculating field offset for account 'Mint' field 'name'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'name' at offset 73
DSL Compiler DEBUG: FieldAssignment for account 'mint_account', mapped to index 1, field_offset 73
    FieldAssignment details: object=Identifier("mint_account"), field="symbol"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: symbol
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 6 for parameter 'symbol'
AST Generator: Calculating field offset for account 'Mint' field 'symbol'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'symbol' at offset 105
DSL Compiler DEBUG: FieldAssignment for account 'mint_account', mapped to index 1, field_offset 105
    FieldAssignment details: object=Identifier("mint_account"), field="uri"
AST Generator DEBUG: Processing node: "FieldAssignment"
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
    Identifier: uri
AST Generator DEBUG: Processing node: "Identifier"
DEBUG: Generated LOAD_PARAM 7 for parameter 'uri'
AST Generator: Calculating field offset for account 'Mint' field 'uri'
AST Generator: Account registry has 2 registered types
AST Generator: Registry contains account type: 'TokenAccount'
AST Generator: Registry contains account type: 'Mint'
AST Generator: Found account type 'Mint' with 7 fields
AST Generator: Account 'Mint' has field 'freeze_authority' at offset 32
AST Generator: Account 'Mint' has field 'decimals' at offset 72
AST Generator: Account 'Mint' has field 'name' at offset 73
AST Generator: Account 'Mint' has field 'symbol' at offset 105
AST Generator: Account 'Mint' has field 'authority' at offset 0
AST Generator: Account 'Mint' has field 'uri' at offset 137
AST Generator: Account 'Mint' has field 'supply' at offset 64
AST Generator: Found field 'uri' at offset 137
DSL Compiler DEBUG: FieldAssignment for account 'mint_account', mapped to index 1, field_offset 137
AST Generator DEBUG: Processing node: "Other"
AST Generator DEBUG: Processing node: "Other"
AST Generator: Processing FieldAccess for 'mint_account' field 'key'
AST Generator: Found symbol 'mint_account' with type 'Mint'
AST Generator: Using built-in property access for 'mint_account.key'
DEBUG: account_index_from_param_index(0) with offset 1 -> 1
DEBUG: Completed AST node generation for function: init_mint
DEBUG: Function init_mint has return type, using RETURN_VALUE from explicit return statement
DEBUG: Completed function body generation for: init_mint
DEBUG: Production optimization - using OptimizedHeader, no metadata patching needed
✓ Bytecode generation successful!
  Bytecode length: 1009 bytes
  Bytecode (hex): 354956450f0100000e0eb1010e09696e69745f6d696e7412696e69745f746f6b656e5f6163636f756e74076d696e745f746f087472616e736665720d7472616e736665725f66726f6d07617070726f7665067265766f6b65046275726e0e667265657a655f6163636f756e740c746861775f6163636f756e74127365745f6d696e745f617574686f72697479147365745f667265657a655f617574686f726974790c64697361626c655f6d696e740e64697361626c655f667265657a65dc190027022001dc190127023401dc190227024001dc190327024e01dc190427025c01dc190527026a01dc190627027801dc190727028201dc190827029001dc190927029c01dc190a2702a801dc190b2702b401dc190c2702c001dc190d2702ca0100a501a502a503a504a505a506a5079007b6030000a501a502a503900310020000a501a502a503a504900497020000a501a502a503a5049004c8020000a501a502a503a504900406030000a501a502a503a504900463030000a501a5029002d8010000a501a502a503a50490047a030000a501a502a503900345020000a501a502a50390035d020000a501a502a503900375020000a501a502a503900385020000a501a5029002ec010000a501a5029002fc0100007101700248010057022704d8420151d8420149067101700248010057022704d8420100067101700248012057022704d84201200670027101700170027501d81bc0018318021bc0011801845702420100df420120d84201401d00420148d8420149d84201511d014201715701077102700348012057032704480220570127041d01420248067102700348012057032704480220570127041d00420248067101700248010057022704df420100067101700248012057022704df4201200671017102700348010057032704480220570127044302483204a504d82504430140a50420420140430240a504204202400671017102700348010057032704430140a5042804480120480220270443014832044302483204a504d82504430140a50421420140430240a5042042024006710171027003480100570327d4d03203200348015157032704430149a5042804430140a5042804480120480220270443014832044302483204a504d82504d032034c03430149a50421420149430140a50421420140430240a50420420240067101700248010057022704df420151a5044201490671017102700348020057032704430240a5042804480220570127044302483204a504d82504430140a50421420140430240a5042142024006700271017001710270027501d81b80028318021b8002180184a504181429045702420100df420120d8420140a504420148a505420149a506420169a5074201890157010700

  Disassembly:
Disassembly relative to offset 10:
  000a: b1 LOAD_REG_U32 1
  000c: 0e UNKNOWN 
  000d: 09 BR_EQ_U8 105
  000f: 6e UNKNOWN 
  0010: 69 UNKNOWN 
  0011: 74 CHECK_PDA 95
  0013: 6d UNKNOWN 
  0014: 69 UNKNOWN 
  0015: 6e UNKNOWN 
  0016: 74 CHECK_PDA 18
  0018: 69 UNKNOWN 
  0019: 6e UNKNOWN 
  001a: 69 UNKNOWN 
  001b: 74 CHECK_PDA 95
  001d: 74 CHECK_PDA 111
  001f: 6b UNKNOWN 
  0020: 65 ARRAY_GET 
  0021: 6e UNKNOWN 
  0022: 5f UNKNOWN 
  0023: 61 PUSH_ARRAY_LITERAL 99
  0025: 63 ARRAY_LENGTH 
  0026: 6f UNKNOWN 
  0027: 75 CHECK_UNINITIALIZED 110
  0029: 74 CHECK_PDA 7
  002b: 6d UNKNOWN 
  002c: 69 UNKNOWN 
  002d: 6e UNKNOWN 
  002e: 74 CHECK_PDA 95
  0030: 74 CHECK_PDA 111
  0032: 08 UNKNOWN 
  0033: 74 CHECK_PDA 114
  0035: 61 PUSH_ARRAY_LITERAL 110
  0037: 73 CHECK_INITIALIZED 102
  0039: 65 ARRAY_GET 
  003a: 72 CHECK_OWNER 13
  003c: 74 CHECK_PDA 114
  003e: 61 PUSH_ARRAY_LITERAL 110
  0040: 73 CHECK_INITIALIZED 102
  0042: 65 ARRAY_GET 
  0043: 72 CHECK_OWNER 95
  0045: 66 PUSH_STRING_LITERAL 114
  0047: 6f UNKNOWN 
  0048: 6d UNKNOWN 
  0049: 07 RETURN_VALUE 
  004a: 61 PUSH_ARRAY_LITERAL 112
  004c: 70 CHECK_SIGNER 114
  004e: 6f UNKNOWN 
  004f: 76 UNKNOWN 
  0050: 65 ARRAY_GET 
  0051: 06 RETURN 
  0052: 72 CHECK_OWNER 101
  0054: 76 UNKNOWN 
  0055: 6f UNKNOWN 
  0056: 6b UNKNOWN 
  0057: 65 ARRAY_GET 
  0058: 04 REQUIRE 
  0059: 62 ARRAY_INDEX 
  005a: 75 CHECK_UNINITIALIZED 114
  005c: 6e UNKNOWN 
  005d: 0e UNKNOWN 
  005e: 66 PUSH_STRING_LITERAL 114
  0060: 65 ARRAY_GET 
  0061: 65 ARRAY_GET 
  0062: 7a UNKNOWN 
  0063: 65 ARRAY_GET 
  0064: 5f UNKNOWN 
  0065: 61 PUSH_ARRAY_LITERAL 99
  0067: 63 ARRAY_LENGTH 
  0068: 6f UNKNOWN 
  0069: 75 CHECK_UNINITIALIZED 110
  006b: 74 CHECK_PDA 12
  006d: 74 CHECK_PDA 104
  006f: 61 PUSH_ARRAY_LITERAL 119
  0071: 5f UNKNOWN 
  0072: 61 PUSH_ARRAY_LITERAL 99
  0074: 63 ARRAY_LENGTH 
  0075: 6f UNKNOWN 
  0076: 75 CHECK_UNINITIALIZED 110
  0078: 74 CHECK_PDA 18
  007a: 73 CHECK_INITIALIZED 101
  007c: 74 CHECK_PDA 95
  007e: 6d UNKNOWN 
  007f: 69 UNKNOWN 
  0080: 6e UNKNOWN 
  0081: 74 CHECK_PDA 95
  0083: 61 PUSH_ARRAY_LITERAL 117
  0085: 74 CHECK_PDA 104
  0087: 6f UNKNOWN 
  0088: 72 CHECK_OWNER 105
  008a: 74 CHECK_PDA 121
  008c: 14 PICK 
  008d: 73 CHECK_INITIALIZED 101
  008f: 74 CHECK_PDA 95
  0091: 66 PUSH_STRING_LITERAL 114
  0093: 65 ARRAY_GET 
  0094: 65 ARRAY_GET 
  0095: 7a UNKNOWN 
  0096: 65 ARRAY_GET 
  0097: 5f UNKNOWN 
  0098: 61 PUSH_ARRAY_LITERAL 117
  009a: 74 CHECK_PDA 104
  009c: 6f UNKNOWN 
  009d: 72 CHECK_OWNER 105
  009f: 74 CHECK_PDA 121
  00a1: 0c UNKNOWN 
  00a2: 64 ARRAY_SET 
  00a3: 69 UNKNOWN 
  00a4: 73 CHECK_INITIALIZED 97
  00a6: 62 ARRAY_INDEX 
  00a7: 6c UNKNOWN 
  00a8: 65 ARRAY_GET 
  00a9: 5f UNKNOWN 
  00aa: 6d UNKNOWN 
  00ab: 69 UNKNOWN 
  00ac: 6e UNKNOWN 
  00ad: 74 CHECK_PDA 14
  00af: 64 ARRAY_SET 
  00b0: 69 UNKNOWN 
  00b1: 73 CHECK_INITIALIZED 97
  00b3: 62 ARRAY_INDEX 
  00b4: 6c UNKNOWN 
  00b5: 65 ARRAY_GET 
  00b6: 5f UNKNOWN 
  00b7: 66 PUSH_STRING_LITERAL 114
  00b9: 65 ARRAY_GET 
  00ba: 65 ARRAY_GET 
  00bb: 7a UNKNOWN 
  00bc: 65 ARRAY_GET 
  00bd: dc LOAD_PARAM_0 
  00be: 19 PUSH_U16 0
  00c0: 27 EQ 
  00c1: 02 JUMP_IF 32
  00c3: 01 JUMP 3292
  00c6: 01 JUMP 39
  00c8: 02 JUMP_IF 52
  00ca: 01 JUMP 3292
  00cd: 02 JUMP_IF 39
  00cf: 02 JUMP_IF 64
  00d1: 01 JUMP 3292
  00d4: 03 JUMP_IF_NOT 39
  00d6: 02 JUMP_IF 78
  00d8: 01 JUMP 3292
  00db: 04 REQUIRE 
  00dc: 27 EQ 
  00dd: 02 JUMP_IF 92
  00df: 01 JUMP 3292
  00e2: 05 UNKNOWN 
  00e3: 27 EQ 
  00e4: 02 JUMP_IF 106
  00e6: 01 JUMP 3292
  00e9: 06 RETURN 
  00ea: 27 EQ 
  00eb: 02 JUMP_IF 120
  00ed: 01 JUMP 3292
  00f0: 07 RETURN_VALUE 
  00f1: 27 EQ 
  00f2: 02 JUMP_IF 130
  00f5: dc LOAD_PARAM_0 
  00f6: 19 PUSH_U16 8
  00f8: 27 EQ 
  00f9: 02 JUMP_IF 144
  00fc: dc LOAD_PARAM_0 
  00fd: 19 PUSH_U16 9
  00ff: 27 EQ 
  0100: 02 JUMP_IF 156
  0103: dc LOAD_PARAM_0 
  0104: 19 PUSH_U16 10
  0106: 27 EQ 
  0107: 02 JUMP_IF 168
  010a: dc LOAD_PARAM_0 
  010b: 19 PUSH_U16 11
  010d: 27 EQ 
  010e: 02 JUMP_IF 180
  0111: dc LOAD_PARAM_0 
  0112: 19 PUSH_U16 12
  0114: 27 EQ 
  0115: 02 JUMP_IF 192
  0118: dc LOAD_PARAM_0 
  0119: 19 PUSH_U16 13
  011b: 27 EQ 
  011c: 02 JUMP_IF 202
  011f: 00 HALT 
  0120: a5 LOAD_PARAM 1
  0122: a5 LOAD_PARAM 2
  0124: a5 LOAD_PARAM 3
  0126: a5 LOAD_PARAM 4
  0128: a5 LOAD_PARAM 5
  012a: a5 LOAD_PARAM 6
  012c: a5 LOAD_PARAM 7
  012e: 90 CALL params:7 addr:950
  0132: 00 HALT 
  0133: 00 HALT 
  0134: a5 LOAD_PARAM 1
  0136: a5 LOAD_PARAM 2
  0138: a5 LOAD_PARAM 3
  013a: 90 CALL params:3 addr:528
  013e: 00 HALT 
  013f: 00 HALT 
  0140: a5 LOAD_PARAM 1
  0142: a5 LOAD_PARAM 2
  0144: a5 LOAD_PARAM 3
  0146: a5 LOAD_PARAM 4
  0148: 90 CALL params:4 addr:663
  014c: 00 HALT 
  014d: 00 HALT 
  014e: a5 LOAD_PARAM 1
  0150: a5 LOAD_PARAM 2
  0152: a5 LOAD_PARAM 3
  0154: a5 LOAD_PARAM 4
  0156: 90 CALL params:4 addr:712
  015a: 00 HALT 
  015b: 00 HALT 
  015c: a5 LOAD_PARAM 1
  015e: a5 LOAD_PARAM 2
  0160: a5 LOAD_PARAM 3
  0162: a5 LOAD_PARAM 4
  0164: 90 CALL params:4 addr:774
  0168: 00 HALT 
  0169: 00 HALT 
  016a: a5 LOAD_PARAM 1
  016c: a5 LOAD_PARAM 2
  016e: a5 LOAD_PARAM 3
  0170: a5 LOAD_PARAM 4
  0172: 90 CALL params:4 addr:867
  0176: 00 HALT 
  0177: 00 HALT 
  0178: a5 LOAD_PARAM 1
  017a: a5 LOAD_PARAM 2
  017c: 90 CALL params:2 addr:472
  0180: 00 HALT 
  0181: 00 HALT 
  0182: a5 LOAD_PARAM 1
  0184: a5 LOAD_PARAM 2
  0186: a5 LOAD_PARAM 3
  0188: a5 LOAD_PARAM 4
  018a: 90 CALL params:4 addr:890
  018e: 00 HALT 
  018f: 00 HALT 
  0190: a5 LOAD_PARAM 1
  0192: a5 LOAD_PARAM 2
  0194: a5 LOAD_PARAM 3
  0196: 90 CALL params:3 addr:581
  019a: 00 HALT 
  019b: 00 HALT 
  019c: a5 LOAD_PARAM 1
  019e: a5 LOAD_PARAM 2
  01a0: a5 LOAD_PARAM 3
  01a2: 90 CALL params:3 addr:605
  01a6: 00 HALT 
  01a7: 00 HALT 
  01a8: a5 LOAD_PARAM 1
  01aa: a5 LOAD_PARAM 2
  01ac: a5 LOAD_PARAM 3
  01ae: 90 CALL params:3 addr:629
  01b2: 00 HALT 
  01b3: 00 HALT 
  01b4: a5 LOAD_PARAM 1
  01b6: a5 LOAD_PARAM 2
  01b8: a5 LOAD_PARAM 3
  01ba: 90 CALL params:3 addr:645
  01be: 00 HALT 
  01bf: 00 HALT 
  01c0: a5 LOAD_PARAM 1
  01c2: a5 LOAD_PARAM 2
  01c4: 90 CALL params:2 addr:492
  01c8: 00 HALT 
  01c9: 00 HALT 
  01ca: a5 LOAD_PARAM 1
  01cc: a5 LOAD_PARAM 2
  01ce: 90 CALL params:2 addr:508
  01d2: 00 HALT 
  01d3: 00 HALT 
  01d4: 71 CHECK_WRITABLE 1
  01d6: 70 CHECK_SIGNER 2
  01d8: 48 LOAD_FIELD_PUBKEY acc:1 offset:0
  01db: 57 GET_KEY 2
  01dd: 27 EQ 
  01de: 04 REQUIRE 
  01df: d8 PUSH_0 
  01e0: 42 STORE_FIELD acc:1 offset:81
  01e3: d8 PUSH_0 
  01e4: 42 STORE_FIELD acc:1 offset:73
  01e7: 06 RETURN 
  01e8: 71 CHECK_WRITABLE 1
  01ea: 70 CHECK_SIGNER 2
  01ec: 48 LOAD_FIELD_PUBKEY acc:1 offset:0
  01ef: 57 GET_KEY 2
  01f1: 27 EQ 
  01f2: 04 REQUIRE 
  01f3: d8 PUSH_0 
  01f4: 42 STORE_FIELD acc:1 offset:0
  01f7: 06 RETURN 
  01f8: 71 CHECK_WRITABLE 1
  01fa: 70 CHECK_SIGNER 2
  01fc: 48 LOAD_FIELD_PUBKEY acc:1 offset:32
  01ff: 57 GET_KEY 2
  0201: 27 EQ 
  0202: 04 REQUIRE 
  0203: d8 PUSH_0 
  0204: 42 STORE_FIELD acc:1 offset:32
  0207: 06 RETURN 
  0208: 70 CHECK_SIGNER 2
  020a: 71 CHECK_WRITABLE 1
  020c: 70 CHECK_SIGNER 1
  020e: 70 CHECK_SIGNER 2
  0210: 75 CHECK_UNINITIALIZED 1
  0212: d8 PUSH_0 
  0213: 1b PUSH_U64 192
  0216: 83 GET_RENT 
  0217: 18 PUSH_U8 2
  0219: 1b PUSH_U64 192
  021c: 18 PUSH_U8 1
  021e: 84 INIT_ACCOUNT 
  021f: 57 GET_KEY 2
  0221: 42 STORE_FIELD acc:1 offset:0
  0224: df LOAD_PARAM_3 
  0225: 42 STORE_FIELD acc:1 offset:32
  0228: d8 PUSH_0 
  0229: 42 STORE_FIELD acc:1 offset:64
  022c: 1d PUSH_BOOL 
  022d: 00 HALT 
  022e: 42 STORE_FIELD acc:1 offset:72
  0231: d8 PUSH_0 
  0232: 42 STORE_FIELD acc:1 offset:73
  0235: d8 PUSH_0 
  0236: 42 STORE_FIELD acc:1 offset:81
  0239: 1d PUSH_BOOL 
  023a: 01 JUMP 66
  023c: 01 JUMP 113
  023e: 57 GET_KEY 1
  0240: 07 RETURN_VALUE 
  0241: 71 CHECK_WRITABLE 2
  0243: 70 CHECK_SIGNER 3
  0245: 48 LOAD_FIELD_PUBKEY acc:1 offset:32
  0248: 57 GET_KEY 3
  024a: 27 EQ 
  024b: 04 REQUIRE 
  024c: 48 LOAD_FIELD_PUBKEY acc:2 offset:32
  024f: 57 GET_KEY 1
  0251: 27 EQ 
  0252: 04 REQUIRE 
  0253: 1d PUSH_BOOL 
  0254: 01 JUMP 66
  0256: 02 JUMP_IF 72
  0258: 06 RETURN 
  0259: 71 CHECK_WRITABLE 2
  025b: 70 CHECK_SIGNER 3
  025d: 48 LOAD_FIELD_PUBKEY acc:1 offset:32
  0260: 57 GET_KEY 3
  0262: 27 EQ 
  0263: 04 REQUIRE 
  0264: 48 LOAD_FIELD_PUBKEY acc:2 offset:32
  0267: 57 GET_KEY 1
  0269: 27 EQ 
  026a: 04 REQUIRE 
  026b: 1d PUSH_BOOL 
  026c: 00 HALT 
  026d: 42 STORE_FIELD acc:2 offset:72
  0270: 06 RETURN 
  0271: 71 CHECK_WRITABLE 1
  0273: 70 CHECK_SIGNER 2
  0275: 48 LOAD_FIELD_PUBKEY acc:1 offset:0
  0278: 57 GET_KEY 2
  027a: 27 EQ 
  027b: 04 REQUIRE 
  027c: df LOAD_PARAM_3 
  027d: 42 STORE_FIELD acc:1 offset:0
  0280: 06 RETURN 
  0281: 71 CHECK_WRITABLE 1
  0283: 70 CHECK_SIGNER 2
  0285: 48 LOAD_FIELD_PUBKEY acc:1 offset:32
  0288: 57 GET_KEY 2
  028a: 27 EQ 
  028b: 04 REQUIRE 
  028c: df LOAD_PARAM_3 
  028d: 42 STORE_FIELD acc:1 offset:32
  0290: 06 RETURN 
  0291: 71 CHECK_WRITABLE 1
  0293: 71 CHECK_WRITABLE 2
  0295: 70 CHECK_SIGNER 3
  0297: 48 LOAD_FIELD_PUBKEY acc:1 offset:0
  029a: 57 GET_KEY 3
  029c: 27 EQ 
  029d: 04 REQUIRE 
  029e: 48 LOAD_FIELD_PUBKEY acc:2 offset:32
  02a1: 57 GET_KEY 1
  02a3: 27 EQ 
  02a4: 04 REQUIRE 
  02a5: 43 LOAD_FIELD acc:2 offset:72
  02a8: 32 NOT 
  02a9: 04 REQUIRE 
  02aa: a5 LOAD_PARAM 4
  02ac: d8 PUSH_0 
  02ad: 25 GT 
  02ae: 04 REQUIRE 
  02af: 43 LOAD_FIELD acc:1 offset:64
  02b2: a5 LOAD_PARAM 4
  02b4: 20 ADD 
  02b5: 42 STORE_FIELD acc:1 offset:64
  02b8: 43 LOAD_FIELD acc:2 offset:64
  02bb: a5 LOAD_PARAM 4
  02bd: 20 ADD 
  02be: 42 STORE_FIELD acc:2 offset:64
  02c1: 06 RETURN 
  02c2: 71 CHECK_WRITABLE 1
  02c4: 71 CHECK_WRITABLE 2
  02c6: 70 CHECK_SIGNER 3
  02c8: 48 LOAD_FIELD_PUBKEY acc:1 offset:0
  02cb: 57 GET_KEY 3
  02cd: 27 EQ 
  02ce: 04 REQUIRE 
  02cf: 43 LOAD_FIELD acc:1 offset:64
  02d2: a5 LOAD_PARAM 4
  02d4: 28 GTE 
  02d5: 04 REQUIRE 
  02d6: 48 LOAD_FIELD_PUBKEY acc:1 offset:32
  02d9: 48 LOAD_FIELD_PUBKEY acc:2 offset:32
  02dc: 27 EQ 
  02dd: 04 REQUIRE 
  02de: 43 LOAD_FIELD acc:1 offset:72
  02e1: 32 NOT 
  02e2: 04 REQUIRE 
  02e3: 43 LOAD_FIELD acc:2 offset:72
  02e6: 32 NOT 
  02e7: 04 REQUIRE 
  02e8: a5 LOAD_PARAM 4
  02ea: d8 PUSH_0 
  02eb: 25 GT 
  02ec: 04 REQUIRE 
  02ed: 43 LOAD_FIELD acc:1 offset:64
  02f0: a5 LOAD_PARAM 4
  02f2: 21 SUB 
  02f3: 42 STORE_FIELD acc:1 offset:64
  02f6: 43 LOAD_FIELD acc:2 offset:64
  02f9: a5 LOAD_PARAM 4
  02fb: 20 ADD 
  02fc: 42 STORE_FIELD acc:2 offset:64
  02ff: 06 RETURN 
  0300: 71 CHECK_WRITABLE 1
  0302: 71 CHECK_WRITABLE 2
  0304: 70 CHECK_SIGNER 3
  0306: 48 LOAD_FIELD_PUBKEY acc:1 offset:0
  0309: 57 GET_KEY 3
  030b: 27 EQ 
  030c: d4 SET_LOCAL_0 
  030d: d0 GET_LOCAL_0 
  030e: 32 NOT 
  030f: 03 JUMP_IF_NOT 32
  0311: 03 JUMP_IF_NOT 72
  0313: 01 JUMP 81
  0315: 57 GET_KEY 3
  0317: 27 EQ 
  0318: 04 REQUIRE 
  0319: 43 LOAD_FIELD acc:1 offset:73
  031c: a5 LOAD_PARAM 4
  031e: 28 GTE 
  031f: 04 REQUIRE 
  0320: 43 LOAD_FIELD acc:1 offset:64
  0323: a5 LOAD_PARAM 4
  0325: 28 GTE 
  0326: 04 REQUIRE 
  0327: 48 LOAD_FIELD_PUBKEY acc:1 offset:32
  032a: 48 LOAD_FIELD_PUBKEY acc:2 offset:32
  032d: 27 EQ 
  032e: 04 REQUIRE 
  032f: 43 LOAD_FIELD acc:1 offset:72
  0332: 32 NOT 
  0333: 04 REQUIRE 
  0334: 43 LOAD_FIELD acc:2 offset:72
  0337: 32 NOT 
  0338: 04 REQUIRE 
  0339: a5 LOAD_PARAM 4
  033b: d8 PUSH_0 
  033c: 25 GT 
  033d: 04 REQUIRE 
  033e: d0 GET_LOCAL_0 
  033f: 32 NOT 
  0340: 03 JUMP_IF_NOT 76
  0342: 03 JUMP_IF_NOT 67
  0344: 01 JUMP 73
  0346: a5 LOAD_PARAM 4
  0348: 21 SUB 
  0349: 42 STORE_FIELD acc:1 offset:73
  034c: 43 LOAD_FIELD acc:1 offset:64
  034f: a5 LOAD_PARAM 4
  0351: 21 SUB 
  0352: 42 STORE_FIELD acc:1 offset:64
  0355: 43 LOAD_FIELD acc:2 offset:64
  0358: a5 LOAD_PARAM 4
  035a: 20 ADD 
  035b: 42 STORE_FIELD acc:2 offset:64
  035e: 06 RETURN 
  035f: 71 CHECK_WRITABLE 1
  0361: 70 CHECK_SIGNER 2
  0363: 48 LOAD_FIELD_PUBKEY acc:1 offset:0
  0366: 57 GET_KEY 2
  0368: 27 EQ 
  0369: 04 REQUIRE 
  036a: df LOAD_PARAM_3 
  036b: 42 STORE_FIELD acc:1 offset:81
  036e: a5 LOAD_PARAM 4
  0370: 42 STORE_FIELD acc:1 offset:73
  0373: 06 RETURN 
  0374: 71 CHECK_WRITABLE 1
  0376: 71 CHECK_WRITABLE 2
  0378: 70 CHECK_SIGNER 3
  037a: 48 LOAD_FIELD_PUBKEY acc:2 offset:0
  037d: 57 GET_KEY 3
  037f: 27 EQ 
  0380: 04 REQUIRE 
  0381: 43 LOAD_FIELD acc:2 offset:64
  0384: a5 LOAD_PARAM 4
  0386: 28 GTE 
  0387: 04 REQUIRE 
  0388: 48 LOAD_FIELD_PUBKEY acc:2 offset:32
  038b: 57 GET_KEY 1
  038d: 27 EQ 
  038e: 04 REQUIRE 
  038f: 43 LOAD_FIELD acc:2 offset:72
  0392: 32 NOT 
  0393: 04 REQUIRE 
  0394: a5 LOAD_PARAM 4
  0396: d8 PUSH_0 
  0397: 25 GT 
  0398: 04 REQUIRE 
  0399: 43 LOAD_FIELD acc:1 offset:64
  039c: a5 LOAD_PARAM 4
  039e: 21 SUB 
  039f: 42 STORE_FIELD acc:1 offset:64
  03a2: 43 LOAD_FIELD acc:2 offset:64
  03a5: a5 LOAD_PARAM 4
  03a7: 21 SUB 
  03a8: 42 STORE_FIELD acc:2 offset:64
  03ab: 06 RETURN 
  03ac: 70 CHECK_SIGNER 2
  03ae: 71 CHECK_WRITABLE 1
  03b0: 70 CHECK_SIGNER 1
  03b2: 71 CHECK_WRITABLE 2
  03b4: 70 CHECK_SIGNER 2
  03b6: 75 CHECK_UNINITIALIZED 1
  03b8: d8 PUSH_0 
  03b9: 1b PUSH_U64 256
  03bc: 83 GET_RENT 
  03bd: 18 PUSH_U8 2
  03bf: 1b PUSH_U64 256
  03c2: 18 PUSH_U8 1
  03c4: 84 INIT_ACCOUNT 
  03c5: a5 LOAD_PARAM 4
  03c7: 18 PUSH_U8 20
  03c9: 29 LTE 
  03ca: 04 REQUIRE 
  03cb: 57 GET_KEY 2
  03cd: 42 STORE_FIELD acc:1 offset:0
  03d0: df LOAD_PARAM_3 
  03d1: 42 STORE_FIELD acc:1 offset:32
  03d4: d8 PUSH_0 
  03d5: 42 STORE_FIELD acc:1 offset:64
  03d8: a5 LOAD_PARAM 4
  03da: 42 STORE_FIELD acc:1 offset:72
  03dd: a5 LOAD_PARAM 5
  03df: 42 STORE_FIELD acc:1 offset:73
  03e2: a5 LOAD_PARAM 6
  03e4: 42 STORE_FIELD acc:1 offset:105
  03e7: a5 LOAD_PARAM 7
  03e9: 42 STORE_FIELD acc:1 offset:137
  03ed: 57 GET_KEY 1
  03ef: 07 RETURN_VALUE 
  03f0: 00 HALT 

5. ABI GENERATION
--------------------
✓ ABI generation successful!
  Program: Module
  Functions: 14
  Fields: 0
    Function 0: init_mint (index 0)
      - mint_account: Mint (account: true)
      - authority: Account (account: true)
      - freeze_authority: pubkey (account: false)
      - decimals: u8 (account: false)
      - name: string (account: false)
      - symbol: string (account: false)
      - uri: string (account: false)
    Function 1: init_token_account (index 1)
      - token_account: TokenAccount (account: true)
      - owner: Account (account: true)
      - mint: pubkey (account: false)
    Function 2: mint_to (index 2)
      - mint_state: Mint (account: true)
      - destination_account: TokenAccount (account: true)
      - mint_authority: Account (account: true)
      - amount: u64 (account: false)
    Function 3: transfer (index 3)
      - source_account: TokenAccount (account: true)
      - destination_account: TokenAccount (account: true)
      - owner: Account (account: true)
      - amount: u64 (account: false)
    Function 4: transfer_from (index 4)
      - source_account: TokenAccount (account: true)
      - destination_account: TokenAccount (account: true)
      - authority: Account (account: true)
      - amount: u64 (account: false)
    Function 5: approve (index 5)
      - source_account: TokenAccount (account: true)
      - owner: Account (account: true)
      - delegate: pubkey (account: false)
      - amount: u64 (account: false)
    Function 6: revoke (index 6)
      - source_account: TokenAccount (account: true)
      - owner: Account (account: true)
    Function 7: burn (index 7)
      - mint_state: Mint (account: true)
      - source_account: TokenAccount (account: true)
      - owner: Account (account: true)
      - amount: u64 (account: false)
    Function 8: freeze_account (index 8)
      - mint_state: Mint (account: true)
      - account_to_freeze: TokenAccount (account: true)
      - freeze_authority: Account (account: true)
    Function 9: thaw_account (index 9)
      - mint_state: Mint (account: true)
      - account_to_thaw: TokenAccount (account: true)
      - freeze_authority: Account (account: true)
    Function 10: set_mint_authority (index 10)
      - mint_state: Mint (account: true)
      - current_authority: Account (account: true)
      - new_authority: pubkey (account: false)
    Function 11: set_freeze_authority (index 11)
      - mint_state: Mint (account: true)
      - current_freeze_authority: Account (account: true)
      - new_freeze_authority: pubkey (account: false)
    Function 12: disable_mint (index 12)
      - mint_state: Mint (account: true)
      - current_authority: Account (account: true)
    Function 13: disable_freeze (index 13)
      - mint_state: Mint (account: true)
      - current_freeze_authority: Account (account: true)

✓ Bytecode written to: five-templates/token/src/token.bin
✓ ABI written to: five-templates/token/src/token.abi.json
✓ Debug info written to: five-templates/token/src/token.v.debug
