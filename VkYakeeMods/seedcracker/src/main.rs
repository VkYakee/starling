use std::env;
use std::fs;
use std::path::Path;
use std::time::Instant;

use seedcracker::assets_manager::{AssetManager, ASSET_MANAGER, tr};
use seedcracker::bedrock::BedrockSolver;
use seedcracker::biomes::{BiomeGenerator, MC_1_20};
use seedcracker::dungeon::{DungeonFloor, DungeonSolver};
use seedcracker::math::hashed_seed::HashedSeed;
use seedcracker::pillars::EndPillars;
use seedcracker::structures::lifting::StructureLifter;
use seedcracker::structures::{StructureInstance, StructureType};
use seedcracker::timemachine::{CrackerState, TimeMachine};

fn print_banner() {
    println!("========================================================");
    println!("       ███████╗███████╗███████╗██████╗  ██████╗ ");
    println!("       ██╔════╝██╔════╝██╔════╝██╔══██╗██╔════╝ ");
    println!("       ███████╗█████╗  █████╗  ██║  ██║██║      ");
    println!("       ╚════██║██╔══╝  ██╔══╝  ██║  ██║██║      ");
    println!("       ███████║███████╗███████╗██████╔╝╚██████╗ ");
    println!("       ╚══════╝╚══════╝╚══════╝╚═════╝  ╚═════╝ ");
    println!("       SeedCracker Rust Unified Mod & Engine v0.1.0");
    println!("========================================================");
}

fn main() {
    print_banner();

    // Initialize asset manager with Spanish by default
    let cur_dir = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let base_dir = if cur_dir.join("assets").exists() {
        cur_dir
    } else if let Ok(exe_path) = env::current_exe() {
        let manifest_dir = exe_path.ancestors().find(|p| p.join("assets").exists());
        manifest_dir.map(|p| p.to_path_buf()).unwrap_or(cur_dir)
    } else {
        cur_dir
    };
    let mut asset_mgr = AssetManager::init(&base_dir, "es_es");
    if let Ok(mut lock) = ASSET_MANAGER.write() {
        *lock = Some(AssetManager::init(&base_dir, "es_es"));
    }

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "benchmark" => run_benchmark(&base_dir),
        "bedrock" => {
            if args.len() < 3 {
                println!("Uso: seedcracker bedrock <archivo_o_patron.txt> [lista_semillas.txt]");
                return;
            }
            let seedlist_path = if args.len() >= 4 { Some(args[3].as_str()) } else { None };
            run_bedrock(&args[2], seedlist_path);
        }
        "pillars" => {
            if args.len() < 3 {
                println!("Uso: seedcracker pillars <h0> <h1> <h2...> (ejemplo: 76 79 82 85)");
                return;
            }
            run_pillars(&args[2..]);
        }
        "dungeon" => {
            if args.len() < 3 {
                println!("Uso: seedcracker dungeon <patron_suelo> (ejemplo: 01001100110001)");
                return;
            }
            run_dungeon(&args[2]);
        }
        "structures" => {
            if args.len() < 3 {
                println!("Uso: seedcracker structures <tipo:chunkX,chunkZ> ...");
                println!("Tipos válidos: desert, jungle, swamp, igloo, shipwreck, monument, outpost, trial");
                return;
            }
            run_structures(&args[2..]);
        }
        "lift" => {
            if args.len() < 4 {
                println!("Uso: seedcracker lift <structure_seed> --hash <hashed_seed>");
                return;
            }
            run_lift(&args[2..]);
        }
        "locate" => {
            if args.len() < 3 {
                println!("Uso: seedcracker locate <structure> <seed> [centerX] [centerZ]");
                println!("Estructuras: pyramid, temple, swamp, igloo, village, shipwreck, monument, mansion, outpost, city, trial");
                return;
            }
            run_locate(&args[2..]);
        }
        "lang" => {
            if args.len() < 3 {
                println!("Idioma actual: es_es. Disponibles: es_es, en_us, ru_ru, zh_cn, tr_tr, hi_in");
                return;
            }
            asset_mgr.set_locale(&args[2]);
            println!("Idioma cambiado a: {}", args[2]);
            println!("{}", asset_mgr.translate("mod.description"));
        }
        _ => {
            print_usage();
        }
    }
}

fn print_usage() {
    println!("\n{}", tr("cli.usage"));
    println!("{}", tr("cli.available_commands"));
    println!("\nEjemplos:");
    println!("  seedcracker benchmark");
    println!("  seedcracker bedrock assets/examples/seed\\ 765906787396911863.txt");
    println!("  seedcracker pillars 76 79 82 85");
    println!("  seedcracker dungeon 01001100110001");
    println!("  seedcracker structures desert:15,22 swamp:-35,10");
    println!("  seedcracker lift 123456789 --hash -45123984712983719");
}

fn run_benchmark(base_dir: &Path) {
    println!("\n--- [BENCHMARK] Probando motores con datos reales de Nether Bedrock ---");
    let example_file = base_dir.join("assets").join("examples").join("seed 765906787396911863.txt");
    if !example_file.exists() {
        println!("No se encontró el archivo de ejemplo en: {:?}", example_file);
        return;
    }

    let content = fs::read_to_string(&example_file).expect("No se pudo leer archivo de prueba");
    let blocks = BedrockSolver::parse_file(&content);
    println!("Bloques de bedrock cargados del ejemplo: {}", blocks.len());

    let known_seed = 765906787396911863u64;
    println!("Verificando semilla esperada: {}", known_seed);

    let t0 = Instant::now();
    let candidates = vec![known_seed, 12345, 987654321, 555555555];
    let matched = BedrockSolver::crack_from_list(&candidates, &blocks);
    let dt = t0.elapsed();

    println!("Tiempo de filtrado de candidatos: {:?}", dt);
    println!("Semillas coincidentes encontradas: {:?}", matched);
    assert_eq!(matched, vec![known_seed]);
    println!("¡BENCHMARK EXITOSO! El motor verificó con 100% de precisión la semilla original.\n");

    // Test Biome Generator from cubiomes
    println!("--- [BENCHMARK] Probando motor de biomas C cubiomes (1.20) ---");
    let t1 = Instant::now();
    let mut r#gen = BiomeGenerator::new(MC_1_20);
    r#gen.set_seed(known_seed);
    let biome_center = r#gen.get_biome_at_block(0, 64, 0);
    let biome_name = r#gen.get_biome_name(biome_center);
    let dt_biome = t1.elapsed();
    println!("Bioma en (0, 64, 0): ID={} ({}) [en {:?}]", biome_center, biome_name, dt_biome);
}

fn run_bedrock(filepath: &str, seedlist_path: Option<&str>) {
    println!("\n--- [BEDROCK CRACKER] Cargando archivo: {} ---", filepath);
    let content = fs::read_to_string(filepath).expect("Error al leer archivo");
    let blocks = BedrockSolver::parse_file(&content);
    println!("{} bloques cargados.", blocks.len());

    let mut state = CrackerState::new();
    state.bedrock_blocks = blocks;

    if let Some(s_path) = seedlist_path {
        if let Ok(seed_content) = fs::read_to_string(s_path) {
            for line in seed_content.lines() {
                if let Ok(s) = line.trim().parse::<u64>() {
                    state.candidate_seeds.push(s);
                }
            }
            println!("Cargadas {} semillas candidatas desde {}.", state.candidate_seeds.len(), s_path);
        }
    } else {
        let default_list = Path::new("assets/examples/seedlist.txt");
        if default_list.exists() {
            if let Ok(seed_content) = fs::read_to_string(default_list) {
                for line in seed_content.lines() {
                    if let Ok(s) = line.trim().parse::<u64>() {
                        state.candidate_seeds.push(s);
                    }
                }
                println!("Usando lista de semillas candidatas por defecto ({} semillas).", state.candidate_seeds.len());
            }
        }
    }

    let t0 = Instant::now();
    let result = TimeMachine::solve(&state);
    println!("Completado en {:?}", t0.elapsed());
    println!("Semillas encontradas: {:?}", result.bedrock_seeds);
}

fn run_pillars(height_strings: &[String]) {
    println!("\n--- [END PILLARS CRACKER] Analizando alturas ---");
    let mut observed = [None; 10];
    for (i, s) in height_strings.iter().take(10).enumerate() {
        if let Ok(h) = s.parse::<i32>() {
            observed[i] = Some(h);
        }
    }

    let t0 = Instant::now();
    let pillar_seeds = EndPillars::find_pillar_seeds(&observed);
    println!("Completado en {:?}", t0.elapsed());
    println!("Semillas de pilares (16-bit) encontradas ({}): {:?}", pillar_seeds.len(), pillar_seeds);
}

fn run_dungeon(pattern: &str) {
    println!("\n--- [DUNGEON CRACKER] Analizando patrón de suelo ---");
    let calls = DungeonFloor::from_string(pattern);
    let floor = DungeonFloor::new(0, 0, 0, 40, 0, 7, 7, calls);

    let t0 = Instant::now();
    let decorator_seeds = DungeonSolver::crack_decorator_seed(&floor, 14);
    println!("Completado en {:?}", t0.elapsed());
    println!("Semillas de decorador encontradas ({}): {:?}", decorator_seeds.len(), decorator_seeds);
}

fn run_structures(structure_specs: &[String]) {
    println!("\n--- [STRUCTURE LIFTER] Analizando estructuras ---");
    let mut list = Vec::new();

    for spec in structure_specs {
        let parts: Vec<&str> = spec.split(':').collect();
        if parts.len() == 2 {
            let st_type = match parts[0].to_lowercase().as_str() {
                "desert" => StructureType::DesertPyramid,
                "jungle" => StructureType::JungleTemple,
                "swamp" => StructureType::SwampHut,
                "igloo" => StructureType::Igloo,
                "shipwreck" => StructureType::Shipwreck,
                "monument" => StructureType::OceanMonument,
                "outpost" => StructureType::PillagerOutpost,
                "trial" => StructureType::TrialChambers,
                _ => StructureType::DesertPyramid,
            };
            let coords: Vec<&str> = parts[1].split(',').collect();
            if coords.len() == 2 {
                if let (Ok(cx), Ok(cz)) = (coords[0].parse::<i32>(), coords[1].parse::<i32>()) {
                    list.push(StructureInstance::new(st_type, cx, cz));
                }
            }
        }
    }

    println!("Estructuras válidas cargadas: {}", list.len());
    let t0 = Instant::now();
    let seeds = StructureLifter::lift_structures(&list);
    println!("Completado en {:?}", t0.elapsed());
    println!("Semillas de estructura 48-bit encontradas ({}):", seeds.len());
    for s in seeds.iter().take(10) {
        println!("  -> {}", s);
    }
}

fn run_lift(args: &[String]) {
    let structure_seed: u64 = args[0].parse().expect("Semilla de estructura inválida");
    if args.len() >= 3 && args[1] == "--hash" {
        let hash: i64 = args[2].parse().expect("Hashed seed inválida");
        println!("\n--- [HASHED SEED LIFTER] Elevando 48-bit a 64-bit con hash ---");
        let t0 = Instant::now();
        let world_seeds = HashedSeed::find_world_seeds_from_hash(structure_seed, hash);
        println!("Completado en {:?}", t0.elapsed());
        println!("Semillas de mundo 64-bit encontradas ({}): {:?}", world_seeds.len(), world_seeds);
    }
}

fn run_locate(args: &[String]) {
    use seedcracker::structures::finder::{CubiomesStructure, StructureLocator};
    let st_str = args[0].to_lowercase();
    let st = match st_str.as_str() {
        "pyramid" | "desert" => CubiomesStructure::DesertPyramid,
        "temple" | "jungle" => CubiomesStructure::JungleTemple,
        "swamp" | "hut" => CubiomesStructure::SwampHut,
        "igloo" => CubiomesStructure::Igloo,
        "village" => CubiomesStructure::Village,
        "shipwreck" | "ship" => CubiomesStructure::Shipwreck,
        "monument" => CubiomesStructure::Monument,
        "mansion" => CubiomesStructure::Mansion,
        "outpost" => CubiomesStructure::Outpost,
        "city" | "ancient" => CubiomesStructure::AncientCity,
        "trial" => CubiomesStructure::TrialChambers,
        _ => CubiomesStructure::DesertPyramid,
    };

    let seed: u64 = args[1].parse().unwrap_or(0);
    let center_x: i32 = if args.len() >= 3 { args[2].parse().unwrap_or(0) } else { 0 };
    let center_z: i32 = if args.len() >= 4 { args[3].parse().unwrap_or(0) } else { 0 };

    println!("\n--- [STRUCTURE LOCATOR] Localizando {:?} para semilla {} ---", st, seed);
    let mut r#gen = BiomeGenerator::new(MC_1_20);
    r#gen.set_seed(seed);

    let t0 = Instant::now();
    if let Some((x, z, dist)) = StructureLocator::locate_nearest(st, &mut r#gen, seed, center_x, center_z, 10) {
        println!("Completado en {:?}", t0.elapsed());
        println!("Estructura más cercana encontrada en: X={}, Z={} (distancia: {:.1} bloques)", x, z, dist);
    } else {
        println!("No se encontró estructura viable en el radio de búsqueda.");
    }
}
