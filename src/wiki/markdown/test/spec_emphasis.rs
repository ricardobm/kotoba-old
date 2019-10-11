use super::*;

// spell-checker: disable

mod markdown_spec_emphasis {
	use super::*;

	#[test]
	fn example_360() {
		// example 360
		test(
			r##"
				*foo bar*
			"##,
			r##"
				<p><em>foo bar</em></p>
			"##,
		);
	}

	#[test]
	fn example_361() {
		// example 361
		test(
			r##"
				a * foo bar*
			"##,
			r##"
				<p>a * foo bar*</p>
			"##,
		);
	}

	#[test]
	fn example_362() {
		// example 362
		test(
			r##"
				a*"foo"*
			"##,
			r##"
				<p>a*&quot;foo&quot;*</p>
			"##,
		);
	}

	#[test]
	fn example_363() {
		// example 363
		test_raw("*\u{00A0}a\u{00A0}*", "<p>*\u{00A0}a\u{00A0}*</p>");
	}

	#[test]
	fn example_364() {
		// example 364
		test(
			r##"
				foo*bar*
			"##,
			r##"
				<p>foo<em>bar</em></p>
			"##,
		);
	}

	#[test]
	fn example_365() {
		// example 365
		test(
			r##"
				5*6*78
			"##,
			r##"
				<p>5<em>6</em>78</p>
			"##,
		);
	}

	#[test]
	fn example_366() {
		// example 366
		test(
			r##"
				_foo bar_
			"##,
			r##"
				<p><em>foo bar</em></p>
			"##,
		);
	}

	#[test]
	fn example_367() {
		// example 367
		test(
			r##"
				_ foo bar_
			"##,
			r##"
				<p>_ foo bar_</p>
			"##,
		);
	}

	#[test]
	fn example_368() {
		// example 368
		test(
			r##"
				a_"foo"_
			"##,
			r##"
				<p>a_&quot;foo&quot;_</p>
			"##,
		);
	}

	#[test]
	fn example_369() {
		// example 369
		test(
			r##"
				foo_bar_
			"##,
			r##"
				<p>foo_bar_</p>
			"##,
		);
	}

	#[test]
	fn example_370() {
		// example 370
		test(
			r##"
				5_6_78
			"##,
			r##"
				<p>5_6_78</p>
			"##,
		);
	}

	#[test]
	fn example_371() {
		// example 371
		test(
			r##"
				пристаням_стремятся_
			"##,
			r##"
				<p>пристаням_стремятся_</p>
			"##,
		);
	}

	#[test]
	fn example_372() {
		// example 372
		test(
			r##"
				aa_"bb"_cc
			"##,
			r##"
				<p>aa_&quot;bb&quot;_cc</p>
			"##,
		);
	}

	#[test]
	fn example_373() {
		// example 373
		test(
			r##"
				foo-_(bar)_
			"##,
			r##"
				<p>foo-<em>(bar)</em></p>
			"##,
		);
	}

	#[test]
	fn example_374() {
		// example 374
		test(
			r##"
				_foo*
			"##,
			r##"
				<p>_foo*</p>
			"##,
		);
	}

	#[test]
	fn example_375() {
		// example 375
		test(
			r##"
				*foo bar *
			"##,
			r##"
				<p>*foo bar *</p>
			"##,
		);
	}

	#[test]
	fn example_376() {
		// example 376
		test(
			r##"
				*foo bar
				*
			"##,
			r##"
				<p>*foo bar
				*</p>
			"##,
		);
	}

	#[test]
	fn example_377() {
		// example 377
		test(
			r##"
				*(*foo)
			"##,
			r##"
				<p>*(*foo)</p>
			"##,
		);
	}

	#[test]
	fn example_378() {
		// example 378
		test(
			r##"
				*(*foo*)*
			"##,
			r##"
				<p><em>(<em>foo</em>)</em></p>
			"##,
		);
	}

	#[test]
	fn example_379() {
		// example 379
		test(
			r##"
				*foo*bar
			"##,
			r##"
				<p><em>foo</em>bar</p>
			"##,
		);
	}

	#[test]
	fn example_380() {
		// example 380
		test(
			r##"
				_foo bar _
			"##,
			r##"
				<p>_foo bar _</p>
			"##,
		);
	}

	#[test]
	fn example_381() {
		// example 381
		test(
			r##"
				_(_foo)
			"##,
			r##"
				<p>_(_foo)</p>
			"##,
		);
	}

	#[test]
	fn example_382() {
		// example 382
		test(
			r##"
				_(_foo_)_
			"##,
			r##"
				<p><em>(<em>foo</em>)</em></p>
			"##,
		);
	}

	#[test]
	fn example_383() {
		// example 383
		test(
			r##"
				_foo_bar
			"##,
			r##"
				<p>_foo_bar</p>
			"##,
		);
	}

	#[test]
	fn example_384() {
		// example 384
		test(
			r##"
				_пристаням_стремятся
			"##,
			r##"
				<p>_пристаням_стремятся</p>
			"##,
		);
	}

	#[test]
	fn example_385() {
		// example 385
		test(
			r##"
				_foo_bar_baz_
			"##,
			r##"
				<p><em>foo_bar_baz</em></p>
			"##,
		);
	}

	#[test]
	fn example_386() {
		// example 386
		test(
			r##"
				_(bar)_.
			"##,
			r##"
				<p><em>(bar)</em>.</p>
			"##,
		);
	}

	#[test]
	fn example_387() {
		// example 387
		test(
			r##"
				**foo bar**
			"##,
			r##"
				<p><strong>foo bar</strong></p>
			"##,
		);
	}

	#[test]
	fn example_388() {
		// example 388
		test(
			r##"
				** foo bar**
			"##,
			r##"
				<p>** foo bar**</p>
			"##,
		);
	}

	#[test]
	fn example_389() {
		// example 389
		test(
			r##"
				a**"foo"**
			"##,
			r##"
				<p>a**&quot;foo&quot;**</p>
			"##,
		);
	}

	#[test]
	fn example_390() {
		// example 390
		test(
			r##"
				foo**bar**
			"##,
			r##"
				<p>foo<strong>bar</strong></p>
			"##,
		);
	}

	#[test]
	fn example_391() {
		// example 391
		test(
			r##"
				__foo bar__
			"##,
			r##"
				<p><strong>foo bar</strong></p>
			"##,
		);
	}

	#[test]
	fn example_392() {
		// example 392
		test(
			r##"
				__ foo bar__
			"##,
			r##"
				<p>__ foo bar__</p>
			"##,
		);
	}

	#[test]
	fn example_393() {
		// example 393
		test(
			r##"
				__
				foo bar__
			"##,
			r##"
				<p>__
				foo bar__</p>
			"##,
		);
	}

	#[test]
	fn example_394() {
		// example 394
		test(
			r##"
				a__"foo"__
			"##,
			r##"
				<p>a__&quot;foo&quot;__</p>
			"##,
		);
	}

	#[test]
	fn example_395() {
		// example 395
		test(
			r##"
				foo__bar__
			"##,
			r##"
				<p>foo__bar__</p>
			"##,
		);
	}

	#[test]
	fn example_396() {
		// example 396
		test(
			r##"
				5__6__78
			"##,
			r##"
				<p>5__6__78</p>
			"##,
		);
	}

	#[test]
	fn example_397() {
		// example 397
		test(
			r##"
				пристаням__стремятся__
			"##,
			r##"
				<p>пристаням__стремятся__</p>
			"##,
		);
	}

	#[test]
	fn example_398() {
		// example 398
		test(
			r##"
				__foo, __bar__, baz__
			"##,
			r##"
				<p><strong>foo, <strong>bar</strong>, baz</strong></p>
			"##,
		);
	}

	#[test]
	fn example_399() {
		// example 399
		test(
			r##"
				foo-__(bar)__
			"##,
			r##"
				<p>foo-<strong>(bar)</strong></p>
			"##,
		);
	}

	#[test]
	fn example_400() {
		// example 400
		test(
			r##"
				**foo bar **
			"##,
			r##"
				<p>**foo bar **</p>
			"##,
		);
	}

	#[test]
	fn example_401() {
		// example 401
		test(
			r##"
				**(**foo)
			"##,
			r##"
				<p>**(**foo)</p>
			"##,
		);
	}

	#[test]
	fn example_402() {
		// example 402
		test(
			r##"
				*(**foo**)*
			"##,
			r##"
				<p><em>(<strong>foo</strong>)</em></p>
			"##,
		);
	}

	#[test]
	fn example_403() {
		// example 403
		test(
			r##"
				**Gomphocarpus (*Gomphocarpus physocarpus*, syn.
				*Asclepias physocarpa*)**
			"##,
			r##"
				<p><strong>Gomphocarpus (<em>Gomphocarpus physocarpus</em>, syn.
				<em>Asclepias physocarpa</em>)</strong></p>
			"##,
		);
	}

	#[test]
	fn example_404() {
		// example 404
		test(
			r##"
				**foo "*bar*" foo**
			"##,
			r##"
				<p><strong>foo &quot;<em>bar</em>&quot; foo</strong></p>
			"##,
		);
	}

	#[test]
	fn example_405() {
		// example 405
		test(
			r##"
				**foo**bar
			"##,
			r##"
				<p><strong>foo</strong>bar</p>
			"##,
		);
	}

	#[test]
	fn example_406() {
		// example 406
		test(
			r##"
				__foo bar __
			"##,
			r##"
				<p>__foo bar __</p>
			"##,
		);
	}

	#[test]
	fn example_407() {
		// example 407
		test(
			r##"
				__(__foo)
			"##,
			r##"
				<p>__(__foo)</p>
			"##,
		);
	}

	#[test]
	fn example_408() {
		// example 408
		test(
			r##"
				_(__foo__)_
			"##,
			r##"
				<p><em>(<strong>foo</strong>)</em></p>
			"##,
		);
	}

	#[test]
	fn example_409() {
		// example 409
		test(
			r##"
				__foo__bar
			"##,
			r##"
				<p>__foo__bar</p>
			"##,
		);
	}

	#[test]
	fn example_410() {
		// example 410
		test(
			r##"
				__пристаням__стремятся
			"##,
			r##"
				<p>__пристаням__стремятся</p>
			"##,
		);
	}

	#[test]
	fn example_411() {
		// example 411
		test(
			r##"
				__foo__bar__baz__
			"##,
			r##"
				<p><strong>foo__bar__baz</strong></p>
			"##,
		);
	}

	#[test]
	fn example_412() {
		// example 412
		test(
			r##"
				__(bar)__.
			"##,
			r##"
				<p><strong>(bar)</strong>.</p>
			"##,
		);
	}

	#[test]
	fn example_413() {
		// example 413
		test(
			r##"
				*foo [bar](/url)*
			"##,
			r##"
				<p><em>foo <a href="/url">bar</a></em></p>
			"##,
		);
	}

	#[test]
	fn example_414() {
		// example 414
		test(
			r##"
				*foo
				bar*
			"##,
			r##"
				<p><em>foo
				bar</em></p>
			"##,
		);
	}

	#[test]
	fn example_415() {
		// example 415
		test(
			r##"
				_foo __bar__ baz_
			"##,
			r##"
				<p><em>foo <strong>bar</strong> baz</em></p>
			"##,
		);
	}

	#[test]
	fn example_416() {
		// example 416
		test(
			r##"
				_foo _bar_ baz_
			"##,
			r##"
				<p><em>foo <em>bar</em> baz</em></p>
			"##,
		);
	}

	#[test]
	fn example_417() {
		// example 417
		test(
			r##"
				__foo_ bar_
			"##,
			r##"
				<p><em><em>foo</em> bar</em></p>
			"##,
		);
	}

	#[test]
	fn example_418() {
		// example 418
		test(
			r##"
				*foo *bar**
			"##,
			r##"
				<p><em>foo <em>bar</em></em></p>
			"##,
		);
	}

	#[test]
	fn example_419() {
		// example 419
		test(
			r##"
				*foo **bar** baz*
			"##,
			r##"
				<p><em>foo <strong>bar</strong> baz</em></p>
			"##,
		);
	}

	#[test]
	fn example_420() {
		// example 420
		test(
			r##"
				*foo**bar**baz*
			"##,
			r##"
				<p><em>foo<strong>bar</strong>baz</em></p>
			"##,
		);
	}

	#[test]
	fn example_421() {
		// example 421
		test(
			r##"
				*foo**bar*
			"##,
			r##"
				<p><em>foo**bar</em></p>
			"##,
		);
	}

	#[test]
	fn example_422() {
		// example 422
		test(
			r##"
				***foo** bar*
			"##,
			r##"
				<p><em><strong>foo</strong> bar</em></p>
			"##,
		);
	}

	#[test]
	fn example_423() {
		// example 423
		test(
			r##"
				*foo **bar***
			"##,
			r##"
				<p><em>foo <strong>bar</strong></em></p>
			"##,
		);
	}

	#[test]
	fn example_424() {
		// example 424
		test(
			r##"
				*foo**bar***
			"##,
			r##"
				<p><em>foo<strong>bar</strong></em></p>
			"##,
		);
	}

	#[test]
	fn example_425() {
		// example 425
		test(
			r##"
				foo***bar***baz
			"##,
			r##"
				<p>foo<em><strong>bar</strong></em>baz</p>
			"##,
		);
	}

	#[test]
	fn example_426() {
		// example 426
		test(
			r##"
				foo******bar*********baz
			"##,
			r##"
				<p>foo<strong><strong><strong>bar</strong></strong></strong>***baz</p>
			"##,
		);
	}

	#[test]
	fn example_427() {
		// example 427
		test(
			r##"
				*foo **bar *baz* bim** bop*
			"##,
			r##"
				<p><em>foo <strong>bar <em>baz</em> bim</strong> bop</em></p>
			"##,
		);
	}

	#[test]
	fn example_428() {
		// example 428
		test(
			r##"
				*foo [*bar*](/url)*
			"##,
			r##"
				<p><em>foo <a href="/url"><em>bar</em></a></em></p>
			"##,
		);
	}

	#[test]
	fn example_429() {
		// example 429
		test(
			r##"
				** is not an empty emphasis
			"##,
			r##"
				<p>** is not an empty emphasis</p>
			"##,
		);
	}

	#[test]
	fn example_430() {
		// example 430
		test(
			r##"
				**** is not an empty strong emphasis
			"##,
			r##"
				<p>**** is not an empty strong emphasis</p>
			"##,
		);
	}

	#[test]
	fn example_431() {
		// example 431
		test(
			r##"
				**foo [bar](/url)**
			"##,
			r##"
				<p><strong>foo <a href="/url">bar</a></strong></p>
			"##,
		);
	}

	#[test]
	fn example_432() {
		// example 432
		test(
			r##"
				**foo
				bar**
			"##,
			r##"
				<p><strong>foo
				bar</strong></p>
			"##,
		);
	}

	#[test]
	fn example_433() {
		// example 433
		test(
			r##"
				__foo _bar_ baz__
			"##,
			r##"
				<p><strong>foo <em>bar</em> baz</strong></p>
			"##,
		);
	}

	#[test]
	fn example_434() {
		// example 434
		test(
			r##"
				__foo __bar__ baz__
			"##,
			r##"
				<p><strong>foo <strong>bar</strong> baz</strong></p>
			"##,
		);
	}

	#[test]
	fn example_435() {
		// example 435
		test(
			r##"
				____foo__ bar__
			"##,
			r##"
				<p><strong><strong>foo</strong> bar</strong></p>
			"##,
		);
	}

	#[test]
	fn example_436() {
		// example 436
		test(
			r##"
				**foo **bar****
			"##,
			r##"
				<p><strong>foo <strong>bar</strong></strong></p>
			"##,
		);
	}

	#[test]
	fn example_437() {
		// example 437
		test(
			r##"
				**foo *bar* baz**
			"##,
			r##"
				<p><strong>foo <em>bar</em> baz</strong></p>
			"##,
		);
	}

	#[test]
	fn example_438() {
		// example 438
		test(
			r##"
				**foo*bar*baz**
			"##,
			r##"
				<p><strong>foo<em>bar</em>baz</strong></p>
			"##,
		);
	}

	#[test]
	fn example_439() {
		// example 439
		test(
			r##"
				***foo* bar**
			"##,
			r##"
				<p><strong><em>foo</em> bar</strong></p>
			"##,
		);
	}

	#[test]
	fn example_440() {
		// example 440
		test(
			r##"
				**foo *bar***
			"##,
			r##"
				<p><strong>foo <em>bar</em></strong></p>
			"##,
		);
	}

	#[test]
	fn example_441() {
		// example 441
		test(
			r##"
				**foo *bar **baz**
				bim* bop**
			"##,
			r##"
				<p><strong>foo <em>bar <strong>baz</strong>
				bim</em> bop</strong></p>
			"##,
		);
	}

	#[test]
	fn example_442() {
		// example 442
		test(
			r##"
				**foo [*bar*](/url)**
			"##,
			r##"
				<p><strong>foo <a href="/url"><em>bar</em></a></strong></p>
			"##,
		);
	}

	#[test]
	fn example_443() {
		// example 443
		test(
			r##"
				__ is not an empty emphasis
			"##,
			r##"
				<p>__ is not an empty emphasis</p>
			"##,
		);
	}

	#[test]
	fn example_444() {
		// example 444
		test(
			r##"
				____ is not an empty strong emphasis
			"##,
			r##"
				<p>____ is not an empty strong emphasis</p>
			"##,
		);
	}

	#[test]
	fn example_445() {
		// example 445
		test(
			r##"
				foo ***
			"##,
			r##"
				<p>foo ***</p>
			"##,
		);
	}

	#[test]
	fn example_446() {
		// example 446
		test(
			r##"
				foo *\**
			"##,
			r##"
				<p>foo <em>*</em></p>
			"##,
		);
	}

	#[test]
	fn example_447() {
		// example 447
		test(
			r##"
				foo *_*
			"##,
			r##"
				<p>foo <em>_</em></p>
			"##,
		);
	}

	#[test]
	fn example_448() {
		// example 448
		test(
			r##"
				foo *****
			"##,
			r##"
				<p>foo *****</p>
			"##,
		);
	}

	#[test]
	fn example_449() {
		// example 449
		test(
			r##"
				foo **\***
			"##,
			r##"
				<p>foo <strong>*</strong></p>
			"##,
		);
	}

	#[test]
	fn example_450() {
		// example 450
		test(
			r##"
				foo **_**
			"##,
			r##"
				<p>foo <strong>_</strong></p>
			"##,
		);
	}

	#[test]
	fn example_451() {
		// example 451
		test(
			r##"
				**foo*
			"##,
			r##"
				<p>*<em>foo</em></p>
			"##,
		);
	}

	#[test]
	fn example_452() {
		// example 452
		test(
			r##"
				*foo**
			"##,
			r##"
				<p><em>foo</em>*</p>
			"##,
		);
	}

	#[test]
	fn example_453() {
		// example 453
		test(
			r##"
				***foo**
			"##,
			r##"
				<p>*<strong>foo</strong></p>
			"##,
		);
	}

	#[test]
	fn example_454() {
		// example 454
		test(
			r##"
				****foo*
			"##,
			r##"
				<p>***<em>foo</em></p>
			"##,
		);
	}

	#[test]
	fn example_455() {
		// example 455
		test(
			r##"
				**foo***
			"##,
			r##"
				<p><strong>foo</strong>*</p>
			"##,
		);
	}

	#[test]
	fn example_456() {
		// example 456
		test(
			r##"
				*foo****
			"##,
			r##"
				<p><em>foo</em>***</p>
			"##,
		);
	}

	#[test]
	fn example_457() {
		// example 457
		test(
			r##"
				foo ___
			"##,
			r##"
				<p>foo ___</p>
			"##,
		);
	}

	#[test]
	fn example_458() {
		// example 458
		test(
			r##"
				foo _\__
			"##,
			r##"
				<p>foo <em>_</em></p>
			"##,
		);
	}

	#[test]
	fn example_459() {
		// example 459
		test(
			r##"
				foo _*_
			"##,
			r##"
				<p>foo <em>*</em></p>
			"##,
		);
	}

	#[test]
	fn example_460() {
		// example 460
		test(
			r##"
				foo _____
			"##,
			r##"
				<p>foo _____</p>
			"##,
		);
	}

	#[test]
	fn example_461() {
		// example 461
		test(
			r##"
				foo __\___
			"##,
			r##"
				<p>foo <strong>_</strong></p>
			"##,
		);
	}

	#[test]
	fn example_462() {
		// example 462
		test(
			r##"
				foo __*__
			"##,
			r##"
				<p>foo <strong>*</strong></p>
			"##,
		);
	}

	#[test]
	fn example_463() {
		// example 463
		test(
			r##"
				__foo_
			"##,
			r##"
				<p>_<em>foo</em></p>
			"##,
		);
	}

	#[test]
	fn example_464() {
		// example 464
		test(
			r##"
				_foo__
			"##,
			r##"
				<p><em>foo</em>_</p>
			"##,
		);
	}

	#[test]
	fn example_465() {
		// example 465
		test(
			r##"
				___foo__
			"##,
			r##"
				<p>_<strong>foo</strong></p>
			"##,
		);
	}

	#[test]
	fn example_466() {
		// example 466
		test(
			r##"
				____foo_
			"##,
			r##"
				<p>___<em>foo</em></p>
			"##,
		);
	}

	#[test]
	fn example_467() {
		// example 467
		test(
			r##"
				__foo___
			"##,
			r##"
				<p><strong>foo</strong>_</p>
			"##,
		);
	}

	#[test]
	fn example_468() {
		// example 468
		test(
			r##"
				_foo____
			"##,
			r##"
				<p><em>foo</em>___</p>
			"##,
		);
	}

	#[test]
	fn example_469() {
		// example 469
		test(
			r##"
				**foo**
			"##,
			r##"
				<p><strong>foo</strong></p>
			"##,
		);
	}

	#[test]
	fn example_470() {
		// example 470
		test(
			r##"
				*_foo_*
			"##,
			r##"
				<p><em><em>foo</em></em></p>
			"##,
		);
	}

	#[test]
	fn example_471() {
		// example 471
		test(
			r##"
				__foo__
			"##,
			r##"
				<p><strong>foo</strong></p>
			"##,
		);
	}

	#[test]
	fn example_472() {
		// example 472
		test(
			r##"
				_*foo*_
			"##,
			r##"
				<p><em><em>foo</em></em></p>
			"##,
		);
	}

	#[test]
	fn example_473() {
		// example 473
		test(
			r##"
				****foo****
			"##,
			r##"
				<p><strong><strong>foo</strong></strong></p>
			"##,
		);
	}

	#[test]
	fn example_474() {
		// example 474
		test(
			r##"
				____foo____
			"##,
			r##"
				<p><strong><strong>foo</strong></strong></p>
			"##,
		);
	}

	#[test]
	fn example_475() {
		// example 475
		test(
			r##"
				******foo******
			"##,
			r##"
				<p><strong><strong><strong>foo</strong></strong></strong></p>
			"##,
		);
	}

	#[test]
	fn example_476() {
		// example 476
		test(
			r##"
				***foo***
			"##,
			r##"
				<p><em><strong>foo</strong></em></p>
			"##,
		);
	}

	#[test]
	fn example_477() {
		// example 477
		test(
			r##"
				_____foo_____
			"##,
			r##"
				<p><em><strong><strong>foo</strong></strong></em></p>
			"##,
		);
	}

	#[test]
	fn example_478() {
		// example 478
		test(
			r##"
				*foo _bar* baz_
			"##,
			r##"
				<p><em>foo _bar</em> baz_</p>
			"##,
		);
	}

	#[test]
	fn example_479() {
		// example 479
		test(
			r##"
				*foo __bar *baz bim__ bam*
			"##,
			r##"
				<p><em>foo <strong>bar *baz bim</strong> bam</em></p>
			"##,
		);
	}

	#[test]
	fn example_480() {
		// example 480
		test(
			r##"
				**foo **bar baz**
			"##,
			r##"
				<p>**foo <strong>bar baz</strong></p>
			"##,
		);
	}

	#[test]
	fn example_481() {
		// example 481
		test(
			r##"
				*foo *bar baz*
			"##,
			r##"
				<p>*foo <em>bar baz</em></p>
			"##,
		);
	}

	#[test]
	fn example_482() {
		// example 482
		test(
			r##"
				*[bar*](/url)
			"##,
			r##"
				<p>*<a href="/url">bar*</a></p>
			"##,
		);
	}

	#[test]
	fn example_483() {
		// example 483
		test(
			r##"
				_foo [bar_](/url)
			"##,
			r##"
				<p>_foo <a href="/url">bar_</a></p>
			"##,
		);
	}

	#[test]
	fn example_484() {
		// example 484
		test(
			r##"
				*<img src="foo" title="*"/>
			"##,
			r##"
				<p>*<img src="foo" title="*"/></p>
			"##,
		);
	}

	#[test]
	fn example_485() {
		// example 485
		test(
			r##"
				**<a href="**">
			"##,
			r##"
				<p>**<a href="**"></p>
			"##,
		);
	}

	#[test]
	fn example_486() {
		// example 486
		test(
			r##"
				__<a href="__">
			"##,
			r##"
				<p>__<a href="__"></p>
			"##,
		);
	}

	#[test]
	fn example_487() {
		// example 487
		test(
			r##"
				*a `*`*
			"##,
			r##"
				<p><em>a <code>*</code></em></p>
			"##,
		);
	}

	#[test]
	fn example_488() {
		// example 488
		test(
			r##"
				_a `_`_
			"##,
			r##"
				<p><em>a <code>_</code></em></p>
			"##,
		);
	}

	#[test]
	fn example_489() {
		// example 489
		test(
			r##"
				**a<http://foo.bar/?q=**>
			"##,
			r##"
				<p>**a<a href="http://foo.bar/?q=**">http://foo.bar/?q=**</a></p>
			"##,
		);
	}

	#[test]
	fn example_490() {
		// example 490
		test(
			r##"
				__a<http://foo.bar/?q=__>
			"##,
			r##"
				<p>__a<a href="http://foo.bar/?q=__">http://foo.bar/?q=__</a></p>
			"##,
		);
	}

	#[test]
	fn example_491_supports_strikethrough() {
		// example 491
		test(
			r##"
				~~Hi~~ Hello, world!
			"##,
			r##"
				<p><del>Hi</del> Hello, world!</p>
			"##,
		);

		test(
			r##"
				~~Hi~~ Hello, world!
			"##,
			r##"
				<p><del>Hi</del> Hello, world!</p>
			"##,
		);

		test(
			r##"
				__*~~Hi there~~Howdy* partner__!!!
			"##,
			r##"
				<p><strong><em><del>Hi there</del>Howdy</em> partner</strong>!!!</p>
			"##,
		);

		test(
			r##"
				~~__*Hi there*~~ partner__!!!
			"##,
			r##"
				<p><del>__<em>Hi there</em></del> partner__!!!</p>
			"##,
		);
	}

	#[test]
	fn example_491_strikethrough_is_inline() {
		// example 491
		test(
			r##"
				This ~~has a

				new paragraph~~.
			"##,
			r##"
				<p>This ~~has a</p>
				<p>new paragraph~~.</p>
			"##,
		);
	}
}
