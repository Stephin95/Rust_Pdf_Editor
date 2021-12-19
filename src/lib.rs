// cargo run --merge "c:\Users\steph\OneDrive\Documents\University\Financial control\CW3\Class Discussions Financial and Ratio Analysis.pdf" "c:\Users\steph\OneDrive\Documents\University\Financial control\CW3\CW3  - FC - In Depth Analysis of a Companys Performance Using Financial Ratio Analysis.pdf" --out "C:\Users\steph\OneDrive\Documents\University\Financial control\CW3"

pub mod pdf_backend {
    use log::{debug, error, log_enabled, info, Level};
    use lopdf::{Document, Object, ObjectId, Stream,Bookmark};
    use std::collections::BTreeMap;
    use lopdf::content::{Content, Operation};
    

    pub fn clean_args(filepath: &Vec<String>) ->(Vec<String>,String) {
        let mut output_path = String::new();
        let files = &filepath[2..];
        let mut is_out_path = false;
        let mut pdf_files: Vec<String> = Vec::new();
        
        for x in files.into_iter()
        { 
            if x == "--out" {
                is_out_path = true;
                info!("found --out flag");
                // warn!("hi");
                // print!("found --out flag");
                continue;
            }
            if is_out_path {
                // let mut xy= String::new();
                let xy:String =String::from(x);
                output_path =xy;
                println!("read out path");
                info!("out path is {}",output_path);
                is_out_path=false;
                break;
            }
            else {
                // println!("files to merge are {}",String::from(x));
                pdf_files.push(String::from(x));
            }
        }

        // println!("{}", output_path);
        // println!("{:?}",pdf_files);
        // let _documents = vec!["C:\Users\steph\OneDrive\Documents\University\HRM\\HRM_assignment.pdf","C:\\Users\\steph\\OneDrive\\Documents\\University\\HRM\\7aad0062 HRM Coursework 1 AssignmentBriefingSheet_2021-22 final-2.pdf" ];
    return(pdf_files,output_path)
    }
    pub fn load_pdfs(filepath: &Vec<String>)->Vec<Document>
    {
        // let mut cleaned_args:(String, Vec<String>) = Vec::new();
        let cleaned_args=clean_args(filepath);
        info!("{:?}",&cleaned_args);
        let mut documents=Vec::new();
        for path in cleaned_args.0{
            print!("{}",path);
            let mut doc_res = Document::load(&path);
            let mut doc = match doc_res {
                Ok(file) => file,
                Err(error) => panic!("Problem opening the file: {}{:?}", path,error),
            };
            doc.version = "1.5".to_string(); 
            documents.push(doc);
        }
        documents}

    pub fn merge_pdfs(documents:Vec<Document>){
        // Define a starting max_id (will be used as start index for object_ids)
    let mut max_id = 1;
    let mut pagenum = 1;
    // Collect all Documents Objects grouped by a map
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();
    let mut document = Document::with_version("1.5");

    for mut doc in documents {
        let mut first = false;
        doc.renumber_objects_with(max_id);

        max_id = doc.max_id + 1;

        documents_pages.extend(
            doc
                    .get_pages()
                    .into_iter()
                    .map(|(_, object_id)| {
                        if !first {
                            let bookmark = Bookmark::new(String::from(format!("Page_{}", pagenum)), [0.0, 0.0, 1.0], 0, object_id);
                            document.add_bookmark(bookmark, None);
                            first = true;
                            pagenum += 1;
                        }

                        (
                            object_id,
                            doc.get_object(object_id).unwrap().to_owned(),
                        )
                    })
                    .collect::<BTreeMap<ObjectId, Object>>(),
        );
        documents_objects.extend(doc.objects);
    }

    // Catalog and Pages are mandatory
    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    // Process all objects except "Page" type
    for (object_id, object) in documents_objects.iter() {
        // We have to ignore "Page" (as are processed later), "Outlines" and "Outline" objects
        // All other objects should be collected and inserted into the main Document
        match object.type_name().unwrap_or("") {
            "Catalog" => {
                // Collect a first "Catalog" object and use it for the future "Pages"
                catalog_object = Some((
                    if let Some((id, _)) = catalog_object {
                        id
                    } else {
                        *object_id
                    },
                    object.clone(),
                ));
            }
            "Pages" => {
                // Collect and update a first "Pages" object and use it for the future "Catalog"
                // We have also to merge all dictionaries of the old and the new "Pages" object
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref object)) = pages_object {
                        if let Ok(old_dictionary) = object.as_dict() {
                            dictionary.extend(old_dictionary);
                        }
                    }

                    pages_object = Some((
                        if let Some((id, _)) = pages_object {
                            id
                        } else {
                            *object_id
                        },
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            "Page" => {}     // Ignored, processed later and separately
            "Outlines" => {} // Ignored, not supported yet
            "Outline" => {}  // Ignored, not supported yet
            _ => {
                document.objects.insert(*object_id, object.clone());
            }
        }
    }

    // If no "Pages" found abort
    if pages_object.is_none() {
        println!("Pages root not found.");

        return;
    }

    // Iter over all "Page" and collect with the parent "Pages" created before
    for (object_id, object) in documents_pages.iter() {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_object.as_ref().unwrap().0);

            document
                    .objects
                    .insert(*object_id, Object::Dictionary(dictionary));
        }
    }

    // If no "Catalog" found abort
    if catalog_object.is_none() {
        println!("Catalog root not found.");

        return;
    }

    let catalog_object = catalog_object.unwrap();
    let pages_object = pages_object.unwrap();

    // Build a new "Pages" with updated fields
    if let Ok(dictionary) = pages_object.1.as_dict() {
        let mut dictionary = dictionary.clone();

        // Set new pages count
        dictionary.set("Count", documents_pages.len() as u32);

        // Set new "Kids" list (collected from documents pages) for "Pages"
        dictionary.set(
            "Kids",
            documents_pages
                    .into_iter()
                    .map(|(object_id, _)| Object::Reference(object_id))
                    .collect::<Vec<_>>(),
        );

        document
                .objects
                .insert(pages_object.0, Object::Dictionary(dictionary));
    }

    // Build a new "Catalog" with updated fields
    if let Ok(dictionary) = catalog_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_object.0);
        dictionary.remove(b"Outlines"); // Outlines not supported in merged PDFs

        document
                .objects
                .insert(catalog_object.0, Object::Dictionary(dictionary));
    }

    document.trailer.set("Root", catalog_object.0);

    // Update the max internal ID as wasn't updated before due to direct objects insertion
    document.max_id = document.objects.len() as u32;

    // Reorder all new Document objects
    document.renumber_objects();

     //Set any Bookmarks to the First child if they are not set to a page
    document.adjust_zero_pages();

    //Set all bookmarks to the PDF Object tree then set the Outlines to the Bookmark content map.
    if let Some(n) = document.build_outline() {
        if let Ok(x) = document.get_object_mut(catalog_object.0) {
            if let Object::Dictionary(ref mut dict) = x {
                dict.set("Outlines", Object::Reference(n));
            }
        }
    }

    document.compress();

    // Save the merged PDF
    document.save("merged.pdf").unwrap();
}
    }


