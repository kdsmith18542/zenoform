extends Node3D

@onready var zenoform = $ZenoformNode

func _ready():
    var seed_val = 123
    var chunk_json = zenoform.generate_chunk_json(seed_val, 0, 0, 0, 16, 16)
    var chunk_data = JSON.parse_string(chunk_json)
    
    if chunk_data:
        create_chunk_mesh(chunk_data)
        
        # Mock proof for testing
        var proof_json = JSON.stringify({
            "schema_version": "zenoform.proof.v1",
            "prover": "mock",
            "prover_version": "0.1.0",
            "protocol_version": "zenoform-terrain-v1",
            "public_inputs": {
                "world_id": chunk_data["world_id"],
                "seed_hash": chunk_data["seed_hash"],
                "chunk_coord": chunk_data["chunk_coord"],
                "chunk_size": chunk_data["chunk_size"],
                "module_hash": chunk_data["module_hash"],
                "output_commitment": chunk_data["commitment"]
            },
            "proof": {"format": "mock", "payload": {}}
        })
        
        var is_valid = zenoform.verify_chunk_json(chunk_json, proof_json)
        update_status_visual(is_valid)

func create_chunk_mesh(data):
    var st = SurfaceTool.new()
    st.begin(Mesh.PRIMITIVE_TRIANGLES)
    
    var cells = data["cells"]
    var width = data["chunk_size"]["width"]
    var height = data["chunk_size"]["height"]
    
    # Simple grid mesh generation
    for y in range(height - 1):
        for x in range(width - 1):
            var i0 = y * width + x
            var i1 = y * width + (x + 1)
            var i2 = (y + 1) * width + x
            var i3 = (y + 1) * width + (x + 1)
            
            var v0 = Vector3(x, float(cells[i0]["height"]) / 1000.0, y)
            var v1 = Vector3(x + 1, float(cells[i1]["height"]) / 1000.0, y)
            var v2 = Vector3(x, float(cells[i2]["height"]) / 1000.0, y + 1)
            var v3 = Vector3(x + 1, float(cells[i3]["height"]) / 1000.0, y + 1)
            
            st.add_vertex(v0)
            st.add_vertex(v1)
            st.add_vertex(v2)
            
            st.add_vertex(v1)
            st.add_vertex(v3)
            st.add_vertex(v2)
            
    var mesh = st.commit()
    var mesh_instance = MeshInstance3D.new()
    mesh_instance.mesh = mesh
    add_child(mesh_instance)

func update_status_visual(is_valid):
    var status_overlay = ColorRect.new()
    status_overlay.size = Vector2(200, 40)
    status_overlay.position = Vector2(10, 10)
    add_child(status_overlay)
    
    var label = Label.new()
    label.position = Vector2(10, 10)
    add_child(label)
    
    if is_valid:
        label.text = "VERIFIED"
        status_overlay.color = Color(0, 0.8, 0, 0.7)
        print("Protocol Verified: Chunk matches commitment.")
    else:
        label.text = "FAILED"
        status_overlay.color = Color(0.8, 0, 0, 0.7)
        print("Protocol Failure: Chunk data mismatch.")
