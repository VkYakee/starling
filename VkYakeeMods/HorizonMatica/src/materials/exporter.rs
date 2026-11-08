use super::material_list::MaterialList;

pub struct MaterialListExporter;

impl MaterialListExporter {
    pub fn to_json(list: &MaterialList) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(list)
    }

    pub fn to_csv(list: &MaterialList) -> String {
        let mut out = String::from("Item,Total Count,Stacks,Shulker Boxes,Chests\n");
        for e in &list.entries {
            out.push_str(&format!(
                "\"{}\",{},\"{}\",{:.2},{:.2}\n",
                e.item_name,
                e.count,
                e.formatted_stacks(),
                e.shulkers,
                e.chests
            ));
        }
        out
    }

    pub fn to_table(list: &MaterialList) -> String {
        let mut out = String::new();
        out.push_str(&format!("{:<40} | {:<10} | {:<20} | {:<10}\n", "Item", "Count", "Stacks", "Shulkers"));
        out.push_str(&format!("{:-<40}-+-{:-<10}-+-{:-<20}-+-{:-<10}\n", "", "", "", ""));

        for e in &list.entries {
            out.push_str(&format!(
                "{:<40} | {:<10} | {:<20} | {:<10.1}\n",
                e.item_name,
                e.count,
                e.formatted_stacks(),
                e.shulkers
            ));
        }

        out.push_str(&format!("{:-<40}-+-{:-<10}-+-{:-<20}-+-{:-<10}\n", "", "", "", ""));
        out.push_str(&format!("{:<40} | {:<10} |\n", "TOTAL", list.total_items));
        out
    }
}
