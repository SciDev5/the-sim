pub trait TypedBindGroupInit: Clone + Copy {
    fn label(&self) -> &'static str;
    fn entries(&self) -> Vec<wgpu::BindGroupLayoutEntry>;
}

pub struct BindGroupData<'a>(
    wgpu::BindGroupLayout,
    &'a wgpu::Device,
    Vec<wgpu::BindGroupLayoutEntry>,
);
impl<'a> BindGroupData<'a> {
    fn from_init<Init: TypedBindGroupInit>(device: &'a wgpu::Device, init: Init) -> Self {
        let entries = init.entries();
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(init.label()),
            entries: &entries,
        });
        BindGroupData(layout, device, entries)
    }
    pub fn device(&self) -> &wgpu::Device {
        self.1
    }
    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.0
    }
}

pub trait TypedBindGroupGenInner<'a, Init: TypedBindGroupInit, Data>: Sized {
    fn label(data: &Data) -> &str;
    fn from_bind_group_data<'b: 'a>(init: Init, data: BindGroupData<'b>) -> Self;
    fn bind_group_data(&self) -> &BindGroupData;
    fn entries_from_data<'b>(&self, data: &'b Data) -> Vec<wgpu::BindGroupEntry<'b>>;
}

pub trait TypedBindGroupGen<'a, Init: TypedBindGroupInit, Data>: Sized {
    fn new(device: &'a wgpu::Device, init: Init) -> Self;
    fn create(&self, data: &Data) -> wgpu::BindGroup;
    fn layout(&self) -> &wgpu::BindGroupLayout;
}
impl<'a, Init: TypedBindGroupInit, Data, T: TypedBindGroupGenInner<'a, Init, Data>>
    TypedBindGroupGen<'a, Init, Data> for T
{
    fn create(&self, data: &Data) -> wgpu::BindGroup {
        let bind_group_data = self.bind_group_data();

        bind_group_data
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(Self::label(data)),
                layout: bind_group_data.layout(),
                entries: &self.entries_from_data(data),
            })
    }
    fn new(device: &'a wgpu::Device, init: Init) -> Self {
        Self::from_bind_group_data(init, BindGroupData::from_init(&device, init))
    }
    fn layout(&self) -> &wgpu::BindGroupLayout {
        self.bind_group_data().layout()
    }
}
