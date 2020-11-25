//! # Page table entry
//! Some code borrowed from [Phil Opp's Blog](http://os.phil-opp.com/modifying-page-tables.html)

use crate::memory::Frame;

use super::PhysicalAddress;

//�洢ҳ����Ŀ��Ϣ�Ľṹ
#[repr(packed(8))]
//repr(packed(8))������ǿ�� Rust ���������ݣ��������͵����ݽ������С����������������ڴ��ʹ��Ч�ʣ����ܿ��ܻᵼ�������ĸ����á�
pub struct Entry(u64);

bitflags! {
    pub struct EntryFlags: u64 {//64λ
        const PRESENT =         1;//presentλΪ1����ʾ��Ӧ��ҳ����ҳ���Ѿ����뵽�ڴ��С����Ϊ0��������ʣ���ᷢ��ȱҳ�쳣
        const WRITABLE =        1 << 1;//����д��ҳ��
        const USER_ACCESSIBLE = 1 << 2;//�û�����Ȩ�ޣ����û�����ã���ֻ�����ں�ģʽ�·��ʴ�ҳ��
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;//�Ƿ���û���
        const ACCESSED =        1 << 5;//��ҳ�Ƿ�ʹ�ù�
        const DIRTY =           1 << 6;//swap���̿���ͨ�����λ�������Ƿ�ѡ�����ҳ����н���
        const HUGE_PAGE =       1 << 7;//������ʹ��4k��ҳ����4M�Ĵ�ҳ
        const GLOBAL =          1 << 8;//ȫ���趨��ҳ���Ƿ������е�ַ�ռ��ж�����
        const NO_EXECUTE =      1 << 63;//��ֹ�ڴ�ҳ��ִ�д���
        /*
        9-11λ OS���ɷ���ʹ��
        12-51λ �����ַ  40λ
        52-62λ OS���ɷ���ʹ��
        */
    }
}

pub const ADDRESS_MASK: usize = 0x000f_ffff_ffff_f000;
pub const COUNTER_MASK: u64 = 0x3ff0_0000_0000_0000;
//ADDRESS_MASK����������ȡ��ַ��COUNTER_MASK ��������ȡ����ֵ

impl Entry {
    //�����Ŀ
    pub fn set_zero(&mut self) {
        self.0 = 0;
    }

    //����ҳ����Ŀ�Ƿ������
    pub fn is_unused(&self) -> bool {
        self.0 == (self.0 & COUNTER_MASK)
        /*COUNTER_MASK  0x3ff0_0000_0000_0000
        ��Ч��self.0== 0x3ff0_0000_0000_0000
        */
    }

    //ʹҳ����Ŀ������
    pub fn set_unused(&mut self) {
        self.0 &= COUNTER_MASK;
    }

    //��ȡ��ҳ�ĵ�ַ
    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.0 as usize & ADDRESS_MASK)
        //��ADDRESS_MASK�����ȡ��ַ
    }

    //��ȡ��ǰҳ����Ŀ�ı�־λflags
    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
        //from_bits_truncate �ضϣ�ɾ���͸ñ�־����Ӧ���κ�λ
    }

    //��ȡ����֡
    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.flags().contains(EntryFlags::PRESENT) {//���present�Ƿ���EntryFlags��
            Some(Frame::containing_address(self.address()))//����֡
        } else {
            None
        }
    }

    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        debug_assert!(frame.start_address().get() & !ADDRESS_MASK == 0);
        //assert!�����ڶ��Բ������ʽ�Ƿ�Ϊtrue����debug˵��ֻ���ڵ���ģʽ��ʹ��
        //start_address()��ȡ֡�ĵ�ַ
        //�жϵ�ַ�Ƿ񳬳���Χ
        self.0 = (frame.start_address().get() as u64) | flags.bits() | (self.0 & COUNTER_MASK);
        //��ҳ����Ŀ����Ϣƴ�������������ַ��12-51λ��|ҳ����Ŀ������Ϣ(��12λ��|����ֵ��52-62λ��
    }

    //��ȡ��Ŀ�еĵ�52-61λ������ҳ��ļ�����  10λ
    pub fn counter_bits(&self) -> u64 {
        (self.0 & COUNTER_MASK) >> 52  //��ȡcounter
    }

    //����Ŀ������λ52-61������ҳ��ļ�����
    pub fn set_counter_bits(&mut self, count: u64) {
        self.0 = (self.0 & !COUNTER_MASK) | (count << 52);
        //����(self.0 & !COUNTER_MASK)��52-62λ��������0�����Ž�Ҫ���õ�ֵcount�ƶ���52-62λ���(self.0 & !COUNTER_MASK)ƴ�ӡ�
    }
}

/*
������֤�ǲ��Դ����Ƿ��������ķ�ʽ���е�
��ʹ�� cargo test �������в���ʱ��Rust �ṹ��һ������ִ�г����������ñ���� test ���Եĺ�����������ÿһ��������ͨ������ʧ�ܡ�
*/
#[cfg(test)]
mod tests {
    #[test]
    fn entry_has_required_arch_alignment() {
        use super::Entry;
        assert!(core::mem::align_of::<Entry>() >= core::mem::align_of::<u64>(), "alignment of Entry is less than the required alignment of u64 ({} < {})", core::mem::align_of::<Entry>(), core::mem::align_of::<u64>());
    }
}
